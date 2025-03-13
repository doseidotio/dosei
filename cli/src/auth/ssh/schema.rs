use serde::{Deserialize, Serialize};

pub struct SSH;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SSHBearerPayload {
  pub namespace: String,
  pub nonce: String,
  pub signature: Vec<u8>,
}
