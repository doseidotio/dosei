pub(crate) mod connect;
pub(crate) mod dashboard;
pub(crate) mod default;
pub(crate) mod deploy;
pub(crate) mod login;
pub(crate) mod logs;
pub(crate) mod ls;
pub(crate) mod set_default;

use crate::cluster::Cluster;
use crate::ssh::SSH;
use anyhow::anyhow;
use clap::Subcommand;
use dosei_schema::Dosei;
use std::path::Path;

#[derive(Subcommand)]
pub enum Commands {
  /// SSH into the cluster
  Connect,
  /// Deploy a Dosei Cluster
  Deploy {
    /// Deploy even if the working directory is dirty
    #[arg(long = "allow-dirty")]
    allow_dirty: bool,

    /// Allow using an ip address, which will result in a non SSL API endpoint
    #[arg(long = "allow-invalid-domain")]
    allow_invalid_domain: bool,
  },
  /// Stream Dosei Logs
  Logs,
  /// List of all locally configured clusters
  Ls,
  /// Log in to a cluster
  Login {
    /// Cluster name
    name: Option<String>,

    /// Your cluster username
    username: Option<String>,

    /// Automatically use default SSH key if available
    #[arg(short = 'y', long = "yes")]
    yes: bool,
  },
  /// Get the default cluster
  Default,
  /// Set the default cluster
  SetDefault {
    /// Cluster name to set as default
    name: String,
  },
  /// Log in to the current cluster Dashboard
  #[command(alias = "dash", alias = "view", alias = "console")]
  Dashboard {
    /// Cluster name
    name: Option<String>,
  },
}

impl SSH {
  pub(crate) fn get_credentials_from_cluster(
    cluster: &Cluster,
  ) -> anyhow::Result<(String, String, String)> {
    let server_url = if let Some(servers) = &cluster.servers {
      if servers.is_empty() {
        return Err(anyhow!(
          "Could not find SSH keys. Tried ~/.ssh/id_ed25519 and ~/.ssh/id_rsa"
        ));
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
    Ok((
      username.to_string(),
      hostname.to_string(),
      server_url.to_string(),
    ))
  }
}

impl Cluster {
  pub(crate) fn get_from_dosei_file() -> anyhow::Result<Self> {
    // Find dosei files in current directory
    let dosei_files = dosei_util::dosei_service_configs()?;
    let file_path = dosei_util::find_dosei_file_path(&dosei_files, None)?;
    let dosei_js = Path::new(&file_path);
    Dosei::generate_json_file_from_node(dosei_js)?;
    Cluster::from_json_file()
  }
}
