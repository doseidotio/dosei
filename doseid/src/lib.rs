pub use doseid_macros::Plugin;

pub mod config;
pub mod container;
pub mod job;

use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

#[async_trait::async_trait]
pub trait Plugin: Send + Sync + 'static {
  fn _name(&self) -> &'static str;
  fn _version(&self) -> &'static str;
  async fn _init(&self, config: PluginConfig) -> Result<(), PluginError>;
  async fn _shutdown(&self) -> Result<(), PluginError>;
  fn _as_any(&self) -> &dyn Any;

  // Default implementations that can be overridden
  async fn init(&self, config: PluginConfig) -> Result<(), PluginError> {
    info!(
      "Using default init implementation with config: {:?}",
      config
    );
    Ok(())
  }

  async fn shutdown(&self) -> Result<(), PluginError> {
    info!("Using default shutdown implementation");
    Ok(())
  }
}

#[derive(Clone, Debug)]
pub struct PluginConfig {
  pub settings: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
  #[error("Plugin initialization failed: {0}")]
  InitializationError(String),
  #[error("Plugin operation failed: {0}")]
  OperationError(String),
}

pub struct PluginManager {
  pub plugins: Arc<RwLock<HashMap<String, Arc<dyn Plugin>>>>,
  plugin_dir: PathBuf,
}

impl PluginManager {
  fn get_library_extension() -> &'static str {
    if cfg!(target_os = "macos") {
      "dylib"
    } else if cfg!(target_os = "windows") {
      "dll"
    } else {
      "so"
    }
  }

  pub fn new(plugin_dir: PathBuf) -> Self {
    Self {
      plugins: Arc::new(RwLock::new(HashMap::new())),
      plugin_dir,
    }
  }

  pub async fn load_plugins(&self) -> Result<(), PluginError> {
    // Create plugin directory if it doesn't exist
    if !Path::new(&self.plugin_dir).exists() {
      std::fs::create_dir_all(&self.plugin_dir).map_err(|e| {
        PluginError::InitializationError(format!("Failed to create plugin directory: {}", e))
      })?;
      info!("Created plugin directory: {:?}", self.plugin_dir);
    }
    let entries = std::fs::read_dir(&self.plugin_dir)
      .map_err(|e| PluginError::InitializationError(e.to_string()))?;

    for entry in entries {
      let path = entry
        .map_err(|e| PluginError::InitializationError(e.to_string()))?
        .path();

      if path
        .extension()
        .is_some_and(|ext| ext == Self::get_library_extension())
      {
        info!("Attempting to load plugin from: {:?}", path);
        self.load_plugin(&path).await?;
      } else {
        info!("Skipping non-plugin file: {:?}", path);
      }
    }
    Ok(())
  }

  async fn load_plugin(&self, path: &Path) -> Result<(), PluginError> {
    unsafe {
      let lib = libloading::Library::new(path)
        .map_err(|e| PluginError::InitializationError(e.to_string()))?;

      let constructor: libloading::Symbol<unsafe fn() -> Box<dyn Plugin>> = lib
        .get(b"_plugin_create")
        .map_err(|e| PluginError::InitializationError(e.to_string()))?;

      let plugin = constructor();
      let name = plugin._name().to_string();
      let version = plugin._version().to_string();
      plugin
        ._init(PluginConfig {
          settings: HashMap::new(),
        })
        .await?;
      info!("Loaded {} plugin, version: {}", name, version);

      let mut plugins = self.plugins.write().await;
      // Convert Box<dyn Plugin> to Arc<dyn Plugin>
      plugins.insert(name, Arc::from(plugin));
    }
    Ok(())
  }

  pub async fn get_plugin<T: Plugin + Clone>(&self, name: &str) -> Option<T> {
    let plugins = self.plugins.read().await;
    plugins
      .get(name)
      .and_then(|p| p._as_any().downcast_ref::<T>().cloned())
  }

  pub async fn get_raw_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
    let plugins = self.plugins.read().await;
    plugins.get(name).cloned()
  }
}
