use crate::cluster::{CliClusterInit, Cluster};
use crate::config::{ClusterConfig, Config};
use crate::file::expand_tilde;
use crate::ssh::SSH;
use anyhow::{anyhow, Context};
use colored::Colorize;
use dosei_schema::cluster::ClusterInit;
use ssh2::Session;
use std::io::Write;
use std::net::TcpStream;
use std::path::Path;
use std::{fs, io};

pub fn command(allow_invalid_domain: bool) -> anyhow::Result<()> {
  let cluster = Cluster::get_from_dosei_file()?;

  if !ClusterInit::validate_domain(&cluster.name) {
    eprintln!(
      "{}",
      format!(
        "WARNING: Cluster name '{}' is not a fully qualified domain name.",
        cluster.name
      )
      .yellow()
    );
    if !allow_invalid_domain {
      // Prompt user for confirmation
      eprint!("Do you want to continue anyway? [y/N]: ");
      io::stdout().flush().unwrap();

      let mut input = String::new();
      io::stdin().read_line(&mut input).unwrap();

      if !input.trim().eq_ignore_ascii_case("y") {
        return Err(anyhow!("Deployment cancelled due to invalid domain name"));
      }
    }
  }
  let (username, hostname, server_url) = SSH::get_credentials_from_cluster(&cluster)?;

  let key_path_or_content = if let Some(identity) = cluster.identity {
    identity
  } else {
    SSH::get_default_ssh_key_path()
      .context("Failed to get default ssh key path. Define one")?
      .to_string_lossy()
      .to_string()
  };

  println!("🔐 Connecting to {}...", server_url);

  // Connect to the server
  let tcp =
    TcpStream::connect(format!("{}:22", hostname)).context("Failed to connect to the server")?;

  let mut sess = Session::new().context("Failed to create SSH session")?;
  sess.set_tcp_stream(tcp);
  sess.handshake().context("SSH handshake failed")?;

  if key_path_or_content.contains("-----BEGIN") {
    let temp_file = tempfile::NamedTempFile::new().context("Failed to create temporary file")?;
    fs::write(&temp_file, key_path_or_content).context("Failed to write key to temp file")?;
    sess
      .userauth_pubkey_file(&username, None, temp_file.path(), None)
      .context("Authentication failed")?;
  } else {
    let expanded_path = expand_tilde(&key_path_or_content);
    let private_key_path = Path::new(&expanded_path);
    sess
      .userauth_pubkey_file(&username, None, private_key_path, None)
      .context("Authentication failed")?;
  }

  // Create and serialize the ClusterInit object
  let cluster_init = CliClusterInit(ClusterInit {
    name: cluster.name.clone(),
    dosei_public_key: SSH::generate_ed25519_key()?,
    accounts: cluster.accounts,
  });

  cluster_init.create_lock(&sess)?;
  // Ensure the lock file is removed when function exits, even on error
  let _lock_guard = scopeguard::guard((), |_| {
    if let Err(e) = cluster_init.remove_lock(&sess) {
      eprintln!("Warning: Failed to remove lock file: {}", e);
    }
  });

  cluster_init.save_to_cluster(&sess)?;

  cluster_init.install_docker_on_remote(&sess, &username)?;

  println!("\n🪐 Starting doseid");
  cluster_init.run_doseid_container(&sess)?;

  let mut user_config = Config::load()?;
  let current_dir = std::env::current_dir()?;
  let dosei_dir = current_dir.join(".dosei");
  let private_key_path = dosei_dir.join("dosei_ed25519");
  user_config.add_cluster(
    cluster.name.clone(),
    ClusterConfig {
      id: None,
      username: String::from("dosei"),
      ssh_key: private_key_path.to_str().map(|s| s.to_string()),
    },
  );
  user_config.save()?;

  Ok(())
}
