pub mod git;
pub mod secret;

use anyhow::anyhow;
use dialoguer::Select;
use flate2::write::GzEncoder;
use flate2::Compression;
use ignore::WalkBuilder;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Builder;

/// Create a *.tar.gz from the given path to a target path
///
pub fn write_tar_gz(input_path: &Path, output_path: &Path) -> anyhow::Result<()> {
  let output_path = PathBuf::from(output_path);
  let tar_gz = File::create(output_path)?;
  let enc = GzEncoder::new(tar_gz, Compression::default());
  let mut tar = Builder::new(enc);

  let walker = WalkBuilder::new(input_path)
    .hidden(false)
    .ignore(true)
    .git_ignore(true)
    .git_exclude(true)
    .build();

  for result in walker {
    let entry = result?;
    let path = entry.path();
    if path.is_dir() {
      continue;
    }

    if let Ok(stripped_path) = path.strip_prefix(input_path) {
      tar.append_path_with_name(path, stripped_path)?;
    }
  }

  tar.into_inner()?.finish()?;
  Ok(())
}

pub fn dosei_service_configs() -> anyhow::Result<Vec<DoseiConfig>> {
  let current_dir = std::env::current_dir()?;
  let mut configs = Vec::new();

  if let Ok(entries) = fs::read_dir(current_dir) {
    for entry in entries.filter_map(Result::ok) {
      let path = entry.path();
      if path.is_file() {
        if let Some(stem) = path.file_stem() {
          let stem_str = stem.to_string_lossy();
          if stem_str.starts_with("dosei") {
            if let Some(extension) = path.extension() {
              configs.push(DoseiConfig {
                path: path.clone(),
                extension: extension.to_string_lossy().to_string(),
              });
            }
          }
        }
      }
    }
  }

  if configs.is_empty() {
    return Err(anyhow::Error::msg("No 'dosei.*' files found."));
  }

  Ok(configs)
}

pub fn find_dosei_file_path(
  dosei_files: &[DoseiConfig],
  env_name: Option<String>,
) -> anyhow::Result<PathBuf> {
  let current_dir = std::env::current_dir()?;
  // Find the file path based on rules
  let file_path = if let Some(env) = env_name {
    // If environment is specified, look for the exact matching file
    let env_file_name = format!("dosei.{}.js", env);
    let env_path = current_dir.join(&env_file_name);

    if env_path.exists() {
      env_path
    } else {
      // If the specific environment file doesn't exist, return an error
      return Err(anyhow!("Environment file {} not found", env_file_name));
    }
  } else if dosei_files.len() == 1 {
    // Only one file found, use it
    dosei_files[0].path.clone()
  } else {
    // Multiple files found and no env specified, prompt user to select
    let file_names: Vec<String> = dosei_files
      .iter()
      .filter_map(|p| {
        p.path
          .file_name()
          .and_then(|f| f.to_str())
          .map(String::from)
      })
      .collect();

    let selection = Select::new()
      .with_prompt("Select a dosei configuration file")
      .items(&file_names)
      .default(0)
      .interact()?;

    dosei_files[selection].path.clone()
  };
  println!(
    "⚙️  Using configuration file: {}",
    file_path.file_name().unwrap_or_default().to_string_lossy()
  );
  Ok(file_path)
}

#[derive(Debug)]
pub struct DoseiConfig {
  pub path: PathBuf,
  pub extension: String,
}
