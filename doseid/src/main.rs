mod account;
mod auth;
mod certificate;
mod cluster;
mod config;
mod container;
mod deployment;
mod http;
mod ingress;
mod job;
mod service;
mod session;

use crate::cluster::DaemonClusterInit;
use crate::config::Config;
use crate::container::Container;
use crate::http::Http;
use crate::job::Job;
use anyhow::Context;
use doseid::PluginManager;
use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let config: &'static Config = Box::leak(Box::new(Config::new()?));

  Container::check_docker_daemon_status().await;

  let pg_pool = Pool::<Postgres>::connect(&config.database_url)
    .await
    .context("Failed to connect to Postgres")?;
  sqlx::migrate!().run(&pg_pool).await?;
  let shared_pool = Arc::new(pg_pool);

  let cluster = DaemonClusterInit::new()
    .await
    .context("Cluster creation failed")?;
  cluster
    .init(&shared_pool)
    .await
    .context("Cluster initialization failed")?;

  let plugin_manager = PluginManager::new(PathBuf::from("./plugins"));
  plugin_manager.load_plugins().await?;

  certificate::start_certificate_server(&shared_pool).await?;

  Job::start_server().await?;
  Container::start_event_listener().await?;
  Container::start_monitoring_server().await?;
  Http::start_server(config, &shared_pool).await?;
  Ok(())
}
