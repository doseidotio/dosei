use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct App {
  pub name: Option<String>,
  pub run: Option<String>,
  pub port: Option<u16>,
  pub env: Option<HashMap<String, String>>,
}
