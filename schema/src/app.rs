use crate::DoseiObject;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct AppCronJob {
  pub name: String,
  pub run: String,
  pub is_async: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct App {
  pub name: String,
  pub port: Option<i16>,
  pub domains: Option<Vec<String>>,
  pub env: Option<HashMap<String, String>>,
  pub cron_jobs: Option<Vec<AppCronJob>>,
}

impl DoseiObject for App {
  fn json_path() -> &'static str {
    ".dosei/app.json"
  }
}

impl App {
  pub fn from_string(value: &str) -> anyhow::Result<Self> {
    Ok(serde_json::from_str::<Self>(value)?)
  }
  pub fn from_json_file() -> anyhow::Result<Self> {
    let app_path = Path::new(".").join(Self::json_path());
    let app_data = fs::read_to_string(&app_path)
      .context(format!("Failed to read app file at {:?}", app_path))?;
    Ok(serde_json::from_str::<Self>(&app_data)?)
  }
}
