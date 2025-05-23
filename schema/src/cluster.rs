use crate::DoseiObject;
use regex::Regex;
use serde::{Deserialize, Serialize};

pub const REMOTE_CLUSTER_DOSEI_FOLDER: &str = "~/.dosei";
pub const REMOTE_CLUSTER_POSTGRES_VOLUME: &str = "dosei_postgres";
pub const REMOTE_CLUSTER_DEPLOY_LOCK_FILE: &str = "~/.dosei/deploy.lock";
pub const REMOTE_CLUSTER_DAEMON_FOLDER: &str = "~/.dosei/doseid";
pub const REMOTE_CLUSTER_INIT_FILE: &str = "~/.dosei/doseid/cluster-init.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterAccount {
  pub name: String,
  pub ssh_keys: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterInit {
  pub name: String,
  pub dosei_public_key: String,
  pub accounts: Option<Vec<ClusterAccount>>,
}

impl DoseiObject for ClusterInit {
  fn json_path() -> &'static str {
    ".dosei/cluster.json"
  }
}

impl ClusterInit {
  pub fn validate_domain(domain: &str) -> bool {
    // This regex checks for a valid domain name pattern
    // Domain must have at least one dot and valid characters
    let domain_regex =
      Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$").unwrap();
    domain_regex.is_match(domain)
  }
}
