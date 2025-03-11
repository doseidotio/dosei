use std::fs;
use crate::config::Config;
use anyhow::{anyhow, Context};
use clap::{Arg, ArgMatches, Command};
use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use dialoguer::Select;
use crate::auth::ssh::schema::SSH;
use crate::cluster::schema::{Cluster, ClusterInit};
use crate::file::expand_tilde;

pub fn command() -> Command {
  Command::new("deploy")
    .about("Deploy a Dosei Cluster")
    .about("Check system information and if Docker is installed on a remote server via SSH")
    .arg(
      Arg::new("env")
        .long("env")
        .short('e')
        .help("env file"),
    )
}

pub fn deploy(matches: &ArgMatches, _config: &'static Config) -> anyhow::Result<()> {
  // Get environment from args if provided
  let env_name = matches.get_one::<String>("env");

  // Find dosei files in current directory
  let current_dir = std::env::current_dir()?;
  let entries = fs::read_dir(&current_dir)?;

  // Collect all dosei*.js files
  let mut dosei_files: Vec<PathBuf> = entries
    .filter_map(|entry| {
      let entry = entry.ok()?;
      let path = entry.path();

      if path.is_file() {
        let file_name = path.file_name()?.to_str()?;
        if file_name.starts_with("dosei") && file_name.ends_with(".js") {
          return Some(path);
        }
      }
      None
    })
    .collect();

  // No dosei files found
  if dosei_files.is_empty() {
    return Err(anyhow::anyhow!("No dosei.js files found in current directory"));
  }

  // Find the file path based on rules
  let file_path = if let Some(env) = env_name {
    // If environment is specified, look for the exact matching file
    let env_file_name = format!("dosei.{}.js", env);
    let env_path = current_dir.join(&env_file_name);

    if env_path.exists() {
      env_path
    } else {
      // If the specific environment file doesn't exist, return an error
      return Err(anyhow!("Environment file {} not found", env_file_name));
    }
  } else if dosei_files.len() == 1 {
    // Only one file found, use it
    dosei_files[0].clone()
  } else {
    // Multiple files found and no env specified, prompt user to select
    let file_names: Vec<String> = dosei_files
      .iter()
      .filter_map(|p| p.file_name().and_then(|f| f.to_str()).map(String::from))
      .collect();

    let selection = Select::new()
      .with_prompt("Select a dosei configuration file")
      .items(&file_names)
      .default(0)
      .interact()?;

    dosei_files[selection].clone()
  };

  println!("\n🪐 Deploying Dosei");
  println!("⚙️  Using configuration file: {}", file_path.file_name().unwrap_or_default().to_string_lossy());

  let dosei_js = Path::new(&file_path);
  Cluster::generate_json_file_from_node(dosei_js)?;
  let cluster = Cluster::from_json_file()?;

  let key_path_or_content = if let Some(identity) = cluster.identity {
    identity
  } else {
    SSH::get_default_ssh_key_path()
      .context("Failed to get default ssh key path. Define one")?
      .to_string_lossy()
      .to_string()
  };

  let server_url = if let Some(servers) = &cluster.servers {
    if servers.is_empty() {
      return Err(anyhow!("Could not find SSH keys. Tried ~/.ssh/id_ed25519 and ~/.ssh/id_rsa"))
    }
    servers.first().unwrap()
  } else {
    cluster.name.as_str()
  };
  // Parse server URL (user@hostname)
  let server_url = if !server_url.contains('@') {
    format!("root@{}", server_url)
  } else {
    server_url.to_string()
  };

  let parts: Vec<&str> = server_url.split('@').collect();

  if parts.len() != 2 {
    return Err(anyhow::anyhow!(
        "Invalid server URL format. Expected user@hostname"
    ));
  }

  let username = parts[0];
  let hostname = parts[1];

  println!("🔐 Connecting to {}...", server_url);

  // Connect to the server
  let tcp =
    TcpStream::connect(format!("{}:22", hostname)).context("Failed to connect to the server")?;

  let mut sess = Session::new().context("Failed to create SSH session")?;
  sess.set_tcp_stream(tcp);
  sess.handshake().context("SSH handshake failed")?;

  let dosei_public_key;
  let dosei_pubic_key_email;
  if key_path_or_content.contains("-----BEGIN") {
    let result = SSH::get_public_key_from_private_key(&key_path_or_content)?;
    dosei_public_key = result.0;
    dosei_pubic_key_email = result.1;
    let temp_file = tempfile::NamedTempFile::new().context("Failed to create temporary file")?;
    fs::write(&temp_file, key_path_or_content).context("Failed to write key to temp file")?;
    sess.userauth_pubkey_file(username, None, temp_file.path(), None)
      .context("Authentication failed")?;
  } else {
    let expanded_path = expand_tilde(&key_path_or_content);
    let private_key_path = Path::new(&expanded_path);
    let result = SSH::get_public_key_from_private_key_path(&private_key_path)?;
    dosei_public_key = result.0;
    dosei_pubic_key_email = result.1;
    sess.userauth_pubkey_file(username, None, private_key_path, None)
      .context("Authentication failed")?;
  }

  // Create and serialize the ClusterInit object
  let cluster_init = ClusterInit {
    name: cluster.name.clone(),
    dosei_public_key,
    email: dosei_pubic_key_email,
    accounts: cluster.accounts,
  };

  cluster_init.create_lock(&sess)?;
  // Ensure the lock file is removed when function exits, even on error
  let _lock_guard = scopeguard::guard((), |_| {
    if let Err(e) = cluster_init.remove_lock(&sess) {
      eprintln!("Warning: Failed to remove lock file: {}", e);
    }
  });

  cluster_init.save_to_cluster(&sess)?;

  cluster_init.install_docker_on_remote(&sess, username)?;

  println!("\n🪐 Starting doseid");
  cluster_init.run_doseid_container(&sess)?;

  Ok(())
}
