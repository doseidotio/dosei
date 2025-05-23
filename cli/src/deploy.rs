use crate::cli::Cli;
use crate::config::ApiClient;
use dosei_schema::app::App;
use dosei_schema::Dosei;
use reqwest::blocking::multipart;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs};

pub fn command(cluster_name: Option<String>, allow_dirty: bool) -> anyhow::Result<()> {
  let current_dir = env::current_dir()?;
  let path = Path::new(&current_dir);

  dosei_util::dosei_service_configs()?;

  let output_path = path.join(".dosei/output.tar.gz");
  if let Some(dosei_dir) = output_path.parent() {
    fs::create_dir_all(dosei_dir)?;
  }
  dosei_util::write_tar_gz(path, &output_path)?;

  let app = CliApp::get_from_dosei_file()?;
  let cluster = Cli::get_default_cluster_or_ask(cluster_name)?;

  let base_url = if cluster.0 == "localhost" {
    format!("http://{}", cluster.0)
  } else {
    format!("https://{}", cluster.0)
  };
  let deploy_url = format!("{}/deploy", base_url);
  let mut body = multipart::Form::new();
  body = body.file("file", output_path)?;
  let hash = match dosei_util::git::get_latest_commit_short_hash(path) {
    Ok(hash) => {
      if allow_dirty {
        format!("dirty:{hash}")
      } else {
        hash
      }
    }
    Err(_) => "no-commit".to_string(),
  };
  body = body.text("hash", hash);
  body = body.text("app", serde_json::to_string(&app)?);
  print!("{}", serde_json::to_string_pretty(&app)?);
  let response = ApiClient::default()?
    .post(deploy_url)
    .multipart(body)
    .timeout(Duration::from_secs(3600))
    .bearer_auth(ApiClient::bearer_ssh_token(
      cluster.1.ssh_key.clone().map(PathBuf::from),
    )?)
    .send()?;

  let status_code = response.status();
  if status_code.is_success() {
    println!("⚙️  Deploying...");
    let response_text = response.text()?;
    println!("{}", response_text);
    return Ok(());
  }
  response.error_for_status()?;
  Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CliApp(pub App);

impl Deref for CliApp {
  type Target = App;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl CliApp {
  pub(crate) fn get_from_dosei_file() -> anyhow::Result<Self> {
    let dosei_files = dosei_util::dosei_service_configs()?;
    let file_path = dosei_util::find_dosei_file_path(&dosei_files, None)?;
    let dosei_js = Path::new(&file_path);
    Dosei::generate_json_file_from_node(dosei_js)?;
    Ok(CliApp(App::from_json_file()?))
  }
}
