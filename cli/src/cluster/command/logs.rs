use crate::cluster::Cluster;
use crate::ssh::SSH;
use anyhow::Context;
use std::io::{self, BufRead, Write};
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
  println!(
    "🔍 Streaming logs from container 'doseid' on {}...",
    server_url
  );
  println!("Press Ctrl+C to stop streaming logs");

  // Create an SSH command that will execute docker logs -f doseid remotely
  let mut cmd = Command::new("ssh");
  cmd.arg("-i").arg(&key_path_or_content);
  cmd.arg("-t"); // Force pseudo-terminal allocation to handle signals properly
  cmd.arg(format!("{}@{}", username, hostname));
  cmd.arg("docker logs -f doseid");

  // Configure to stream the output
  cmd.stdout(Stdio::piped());
  cmd.stderr(Stdio::piped());

  // Start the process
  let mut child = cmd
    .spawn()
    .context("Failed to execute SSH command for streaming logs")?;

  // Set up output streaming
  let stdout = child
    .stdout
    .take()
    .context("Failed to capture standard output")?;
  let stderr = child
    .stderr
    .take()
    .context("Failed to capture standard error")?;

  // Setup stdout streaming in a separate thread
  let stdout_thread = std::thread::spawn(move || {
    let mut reader = std::io::BufReader::new(stdout);
    let mut buffer = Vec::new();
    while let Ok(n) = reader.read_until(b'\n', &mut buffer) {
      if n == 0 {
        break;
      }
      io::stdout().write_all(&buffer).unwrap();
      io::stdout().flush().unwrap();
      buffer.clear();
    }
  });

  // Setup stderr streaming in a separate thread
  let stderr_thread = std::thread::spawn(move || {
    let mut reader = io::BufReader::new(stderr);
    let mut buffer = Vec::new();
    while let Ok(n) = reader.read_until(b'\n', &mut buffer) {
      if n == 0 {
        break;
      }
      io::stderr().write_all(&buffer).unwrap();
      io::stderr().flush().unwrap();
      buffer.clear();
    }
  });

  // Wait for the process to complete (Ctrl+C will propagate to the SSH process)
  let status = child.wait().context("Failed to wait for SSH command")?;

  // Wait for output threads to finish
  let _ = stdout_thread.join();
  let _ = stderr_thread.join();

  if !status.success() {
    anyhow::bail!("Log streaming ended with exit code: {}", status);
  }

  println!("\n✅ Log streaming completed");
  Ok(())
}
