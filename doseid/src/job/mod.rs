use std::time::Duration;
use tokio::time::interval;
use tracing::info;

pub struct Job;

impl Job {
  pub async fn start_server() -> anyhow::Result<()> {
    info!("DoseiD Job Server Running");
    tokio::spawn(async move {
      let mut interval = interval(Duration::from_secs(1));
      loop {
        interval.tick().await;

        // let plugins_guard = plugin_manager.plugins.read().await;
        // for (plugin_name, plugin) in plugins_guard.iter() {
        //   if let Err(e) = plugin._shutdown().await {
        //     error!("Failed to run task for plugin {}: {}", plugin_name, e);
        //   }
        // }
      }
    });
    Ok(())
  }
}
