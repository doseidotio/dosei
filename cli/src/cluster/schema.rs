use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
  pub name: String,
  pub ssh_keys: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cluster {
  pub name: String,
  pub servers: Option<Vec<String>>,
  pub identity: Option<String>,
  pub accounts: Option<Vec<Account>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterInit {
  pub name: String,
  pub dosei_public_key: String,
  pub accounts: Option<Vec<Account>>,
}
