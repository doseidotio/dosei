use crate::cli::Cli;
use crate::config::{ApiClient, SessionCredentials};
use anyhow::anyhow;
use std::path::PathBuf;

pub fn command(name: Option<String>) -> anyhow::Result<()> {
  let cluster = Cli::get_default_cluster_or_ask(name)?;
  let base_url = if cluster.0 == "localhost" {
    format!("http://{}", cluster.0)
  } else {
    format!("https://{}", cluster.0)
  };
  println!("{}", base_url);
  let login_url = format!("{}/auth/login/ssh", base_url);
  let response = ApiClient::default()?
    .post(login_url)
    .bearer_auth(ApiClient::bearer_ssh_token(
      cluster.1.ssh_key.clone().map(PathBuf::from),
    )?)
    .send()?;

  let status_code = response.status();
  if status_code.is_success() {
    let session = response.json::<SessionCredentials>()?;
    let base_dashboard_url = if cluster.0 == "localhost" {
      format!("http://{}:8844", cluster.0)
    } else {
      format!("https://{}", cluster.0.replace("api", "dashboard"))
    };
    let dashboard_login_url = format!(
      "{}/login?session={}",
      base_dashboard_url,
      session.to_base64()?
    );
    if webbrowser::open(&dashboard_login_url).is_ok() {
      println!(
        "Your browser has been opened to visit:\n\n{}\n",
        dashboard_login_url
      );
    }
    return Ok(());
  }
  Err(anyhow!("Login Failed!"))
}
