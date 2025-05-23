use crate::config::Config;
use anyhow::anyhow;

pub fn command(name: String) -> anyhow::Result<()> {
  let mut config = Config::load()?;

  // Verify that the cluster exists
  if config.get_cluster(&name).is_none() {
    return Err(anyhow!("Cluster '{}' not found", name));
  }

  // Set as default cluster
  config.default_cluster = Some(name.clone());

  // Save the config
  config.save()?;

  println!("Set '{}' as default cluster", name);
  Ok(())
}
