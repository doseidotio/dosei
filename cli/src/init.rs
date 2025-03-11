use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use git2::Repository;
use std::path::Path;
use std::time::Instant;
use std::{env, fs};
use tempfile::tempdir;

const AVAILABLE_TEMPLATES: [&str; 2] = ["cluster-js", "express"];

pub fn command() -> Command {
  Command::new("init")
    .about("Initialize a new project from a template in an existing directory")
    .long_about(
      r#"
Initialize a new project from a template in an existing directory.
Give a path as an argument to create in the given directory.

If the directory is not already in a git repository, then a new repository is created.
"#,
    )
    .arg(
      Arg::new("template")
        .help("The Template name")
        .index(1)
        .value_parser(AVAILABLE_TEMPLATES),
    )
    .arg(
      Arg::new("path")
        .help("The target directory path")
        .index(2)
        .default_value("."),
    )
}

pub fn init(arg_matches: &ArgMatches) -> anyhow::Result<()> {
  // Get template from args or prompt interactively
  let template = match arg_matches.get_one::<String>("template") {
    Some(t) => t.clone(),
    None => {
      // No template provided, prompt interactively
      let selection = dialoguer::Select::new()
        .with_prompt("Select a template")
        .items(&AVAILABLE_TEMPLATES)
        .default(0)
        .interact()?;

      AVAILABLE_TEMPLATES[selection].to_string()
    }
  };

  let destination = arg_matches.get_one::<String>("path").unwrap();

  println!("Creating a new project from the {} template", &template);

  let temp_dir = tempdir().context("Failed to create temporary directory.")?;
  let temp_path = temp_dir.path();
  let template_path = temp_path.to_path_buf().join(&template);

  let _ = dosei_util::git::clone(
    "https://github.com/doseiai/templates",
    temp_path,
    Some("main"),
  );

  let target_path = env::current_dir()?.join(destination);

  let start_copying = Instant::now();
  copy_dir_all(&template_path, &target_path)?;
  let elapsed = start_copying.elapsed().as_secs_f64() * 1000.0;
  println!("Copying {} completed: {:.2}ms", template, elapsed);

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
    println!("Enter your project directory using: cd {}", &destination);
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
