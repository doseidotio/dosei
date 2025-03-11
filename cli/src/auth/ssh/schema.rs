use serde::{Deserialize, Serialize};

pub struct SSH;

#[derive(Serialize, Deserialize)]
pub struct AuthPayload {
  pub timestamp: String,
  pub nonce: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SSHAuthToken {
  pub payload: String,
  pub signature: String,
}

