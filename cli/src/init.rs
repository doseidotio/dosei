use anyhow::Context;
use clap::ValueEnum;
use git2::Repository;
use std::path::Path;
use std::time::Instant;
use std::{env, fs};
use tempfile::tempdir;

#[derive(Debug, Clone, ValueEnum)]
pub enum InitTemplate {
  #[value(name = "cluster-js")]
  ClusterJs,
  #[value(name = "express")]
  Express,
}

pub fn command(path: String, template: Option<InitTemplate>) -> anyhow::Result<()> {
  // Get template from args or prompt interactively
  let template = template.unwrap_or_else(|| {
    let templates = InitTemplate::value_variants();
    // Convert enum variants to owned strings to avoid the reference issue
    let template_names: Vec<String> = templates
      .iter()
      .map(|t| t.to_possible_value().unwrap().get_name().to_string())
      .collect();

    let selection = dialoguer::Select::new()
      .with_prompt("Select a template")
      .items(&template_names)
      .default(0)
      .interact()
      .unwrap();

    // Convert the selection index to a Template enum variant
    templates[selection].clone()
  });
  let binding = template.to_possible_value().unwrap();
  let template_name = binding.get_name();
  println!(
    "Creating a new project from the {} template",
    &template_name
  );

  let temp_dir = tempdir().context("Failed to create temporary directory.")?;
  let temp_path = temp_dir.path();
  let template_path = temp_path.to_path_buf().join(template_name);

  let _ = dosei_util::git::clone(
    "https://github.com/doseiai/templates",
    temp_path,
    Some("main"),
  );

  let target_path = env::current_dir()?.join(&path);

  let start_copying = Instant::now();
  copy_dir_all(&template_path, &target_path)?;
  let elapsed = start_copying.elapsed().as_secs_f64() * 1000.0;
  println!("Copying {} completed: {:.2}ms", &template_name, elapsed);

  if Repository::open(&target_path).is_err() {
    // No git repository directly at this path
    Repository::init(&target_path)?;
    println!(
      "Initialized empty Git repository in {}",
      &target_path.display()
    );
  }

  println!();
  println!("That's it!");
  if target_path != env::current_dir()? {
    println!("Enter your project directory using: cd {}", &path);
  }
  println!("Join the community at https://discord.gg/BP5aUkhcAh");
  Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
  if !dst.exists() {
    fs::create_dir_all(dst)?;
  }

  for entry in fs::read_dir(src)? {
    let entry = entry?;
    let ty = entry.file_type()?;
    let src_path = entry.path();
    let dst_path = dst.join(entry.file_name());

    if ty.is_dir() {
      copy_dir_all(&src_path, &dst_path)?;
    } else {
      fs::copy(&src_path, &dst_path)?;
    }
  }
  Ok(())
}
