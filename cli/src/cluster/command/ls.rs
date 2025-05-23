use crate::config::Config;
use crate::table::TablePrint;

pub fn command() -> anyhow::Result<()> {
  let config = Config::load()?;
  let clusters = config.list_clusters();

  if clusters.is_empty() {
    println!("No clusters configured");
    return Ok(());
  }

  TablePrint {
    headers: vec![
      "NAME".to_string(),
      "USERNAME".to_string(),
      "SSH KEY".to_string(),
    ],
    rows: clusters
      .iter()
      .map(|(name, cluster)| {
        vec![
          name.clone(),
          cluster.username.clone(),
          cluster
            .ssh_key
            .clone()
            .unwrap_or("default".parse().unwrap()),
        ]
      })
      .collect(),
  }
  .print();

  Ok(())
}
