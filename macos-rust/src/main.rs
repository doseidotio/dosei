use anyhow::{anyhow, Context};
use doseid::job::Job;
use sqlx::{Pool, Postgres};
use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::signal;
use tracing::info;

// Define the explicit Docker path
const DOCKER_PATH: &str = "/usr/local/bin/docker";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Check Docker is available at the explicit path
  check_docker_daemon_status().await?;

  run_doseid_db()?;

  // Wait for Postgres to be ready (try 10 times, waiting 2 seconds between attempts)
  match wait_for_postgres_ready(10, 2).await {
    Ok(_) => {
      // Now it's safe to connect to the database
      let db_url = "postgres://postgres@localhost:8845/postgres";
      let pg_pool = Pool::<Postgres>::connect(db_url)
        .await
        .context("Failed to connect to Postgres")?;

      // Run migrations
      sqlx::migrate!("../doseid").run(&pg_pool).await?;
      let shared_pool = Arc::new(pg_pool);

      println!("✅ Database connection and migrations successful");
    }
    Err(e) => {
      eprintln!("⚠️ Warning: Postgres readiness check failed: {}", e);
      // Decide if you want to continue without database or return error
      // return Err(e);
    }
  };

  println!("\n🪐 Starting doseid");

  Job::start_server().await?;

  signal::ctrl_c()
    .await
    .map_err(|err| anyhow!("Unable to listen for shutdown signal: {}", err))?;

  info!("Gracefully stopping... (Press Ctrl+C again to force)");

  // Stop the doseid-postgres container
  stop_doseid_db()?;

  Ok(())
}

// Custom Docker daemon check using the explicit path
async fn check_docker_daemon_status() -> anyhow::Result<()> {
  // Try to run docker info using the explicit path
  let output = Command::new(DOCKER_PATH)
    .arg("info")
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .output()
    .context("Failed to execute Docker info command")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow!("Docker daemon is not running: {}", stderr.trim()));
  }

  println!("✅ Docker daemon is running");
  Ok(())
}

fn run_doseid_db() -> anyhow::Result<()> {
  // Define container name
  let container_name = "doseid-postgres";

  // Check if the container exists but is stopped - using explicit docker path
  let check_exists_output = Command::new(DOCKER_PATH)
    .args(["ps", "-a", "-q", "-f", &format!("name={}", container_name)])
    .output()
    .context("Failed to check if container exists")?;

  let container_exists = !check_exists_output.stdout.is_empty();

  if container_exists {
    // If container exists, start it if not running
    Command::new(DOCKER_PATH)
      .args(["start", container_name])
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .context("Failed to start existing container")?;
  } else {
    #[allow(deprecated)]
    let folder = format!(
      "{}/.dosei/postgres",
      std::env::home_dir().unwrap().display()
    );
    Command::new(DOCKER_PATH)
      .args([
        "run",
        "-d",
        "-e",
        "POSTGRES_HOST_AUTH_METHOD=trust",
        "-e",
        "POSTGRES_HOST=/var/run/postgresql",
        "-e",
        "PGDATA=/var/lib/postgresql/data/pgdata",
        "-p",
        "8845:5432",
        "-v",
        &format!("{folder}:/var/lib/postgresql/data"),
        "--name",
        container_name,
        "postgres:16",
        "-c",
        "log_connections=on",
        "-c",
        "log_disconnections=on",
        "-c",
        "unix_socket_directories=/var/run/postgresql",
      ])
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .context("Failed to create and run Postgres container")?;
  }
  Ok(())
}

fn stop_doseid_db() -> anyhow::Result<()> {
  // Define container name
  let container_name = "doseid-postgres";

  info!("Stopping Docker container: {}", container_name);

  // Check if the container is running - using explicit docker path
  let check_running_output = Command::new(DOCKER_PATH)
    .args(["ps", "-q", "-f", &format!("name={}", container_name)])
    .output()
    .context("Failed to check if container is running")?;

  let container_running = !check_running_output.stdout.is_empty();

  if container_running {
    // Stop the container - using explicit docker path
    Command::new(DOCKER_PATH)
      .args(["stop", container_name])
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .context("Failed to stop container")?;

    info!("Container {} stopped successfully", container_name);
  } else {
    info!("Container {} is not running", container_name);
  }

  Ok(())
}

// Add this function to check if Postgres is ready
async fn wait_for_postgres_ready(attempts: u32, delay_seconds: u64) -> anyhow::Result<()> {
  println!("Waiting for Postgres to be ready...");

  for attempt in 1..=attempts {
    let output = tokio::process::Command::new(DOCKER_PATH)
      .args(["exec", "doseid-postgres", "pg_isready", "-U", "postgres"])
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .output()
      .await
      .context("Failed to execute pg_isready command")?;

    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      println!("✅ Postgres is ready: {}", stdout.trim());
      return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!(
      "Postgres not ready (attempt {}/{}), waiting {} seconds...",
      attempt, attempts, delay_seconds
    );

    if !stderr.is_empty() {
      println!("Error: {}", stderr.trim());
    }
    if !stdout.is_empty() {
      println!("Output: {}", stdout.trim());
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds)).await;
  }

  Err(anyhow!(
    "Postgres failed to become ready after {} attempts",
    attempts
  ))
}
