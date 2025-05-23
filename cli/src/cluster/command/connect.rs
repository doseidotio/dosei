use crate::cluster::Cluster;
use crate::ssh::SSH;
use anyhow::Context;
use std::process::{Command, Stdio};

pub fn command() -> anyhow::Result<()> {
  let cluster = Cluster::get_from_dosei_file()?;
  let (username, hostname, server_url) = SSH::get_credentials_from_cluster(&cluster)?;

  let key_path_or_content = if let Some(identity) = cluster.identity {
    identity
  } else {
    SSH::get_default_ssh_key_path()
      .context("Failed to get default ssh key path. Define one")?
      .to_string_lossy()
      .to_string()
  };

  println!("🐳 Connecting to container 'doseid' on {}...", server_url);
  println!("Type 'exit' to leave the container");

  // Create an SSH command that will execute docker exec -it doseid bash remotely
  let mut cmd = Command::new("ssh");
  cmd.arg("-i").arg(&key_path_or_content);
  cmd.arg("-t"); // Force pseudo-terminal allocation for interactive session
  cmd.arg(format!("{}@{}", username, hostname));
  cmd.arg("docker exec -it doseid bash");

  // Configure for interactive mode - inherit all stdio
  cmd.stdin(Stdio::inherit());
  cmd.stdout(Stdio::inherit());
  cmd.stderr(Stdio::inherit());

  // Execute the command and wait for completion
  let status = cmd
    .status()
    .context("Failed to execute SSH command for container access")?;

  if !status.success() {
    anyhow::bail!(
      "Container access ended with exit code: {}",
      status.code().unwrap_or(-1)
    );
  }

  println!("\n✅ Container session ended");
  Ok(())
}
