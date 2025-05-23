use crate::config::Config;
use crate::table::TablePrint;

pub fn command() -> anyhow::Result<()> {
  let config = Config::load()?;

  let cluster = match config.get_default_cluster() {
    None => {
      println!("No default cluster configured");
      return Ok(());
    }
    Some(clusters) => clusters,
  };

  TablePrint {
    headers: vec![
      "NAME".to_string(),
      "USERNAME".to_string(),
      "SSH KEY".to_string(),
    ],
    rows: cluster
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
