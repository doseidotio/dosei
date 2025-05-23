mod default;

use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
  pub host: String,
  pub database_url: String,
}

impl Config {
  pub fn new() -> anyhow::Result<Config> {
    // Load env variables from `.env`, if any.
    dotenv().ok();

    // Configure logging
    let subscriber = tracing_subscriber::fmt()
      .with_target(false)
      .with_max_level(tracing::Level::INFO)
      .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(Config {
      host: "0.0.0.0".to_string(),
      database_url: env::var("DATABASE_URL").unwrap_or(default::DATABASE_URL.to_string()),
    })
  }

  pub fn address(&self) -> String {
    format!("{}:{}", self.host, 80)
  }

  pub fn proxy_address(&self) -> String {
    format!("{}:{}", self.host, 443)
  }
}
