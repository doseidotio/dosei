use home::home_dir;
use std::path::{Path, PathBuf};

// Function to expand ~ to home directory
pub fn expand_tilde(path: &str) -> PathBuf {
  if path.starts_with("~/") {
    if let Some(home) = home_dir() {
      return home.join(&path[2..]);
    }
  }
  Path::new(path).to_path_buf()
}
