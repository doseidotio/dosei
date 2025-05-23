use home::home_dir;
use std::path::{Path, PathBuf};

// Function to expand ~ to home directory
pub fn expand_tilde(path: &str) -> PathBuf {
  if let Some(stripped) = path.strip_prefix("~/") {
    if let Some(home) = home_dir() {
      return home.join(stripped);
    }
  }
  Path::new(path).to_path_buf()
}
