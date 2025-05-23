use anyhow::Context;
use std::path::Path;
use std::process::Stdio;

pub mod app;
pub mod cluster;
pub mod ssh;

pub struct Dosei;

impl Dosei {
  pub fn generate_json_file_from_node(path: &Path) -> anyhow::Result<()> {
    std::process::Command::new("node")
      .arg(path)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .context("Failed to read doseid config")?;
    Ok(())
  }
}

pub trait DoseiObject {
  fn json_path() -> &'static str;
}
