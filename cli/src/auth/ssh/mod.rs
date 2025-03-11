use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Context};
use base64::Engine;
use base64::engine::general_purpose;
use chrono::Utc;
use ssh2::Session;
use ssh_key::{HashAlg, PrivateKey};
use uuid::Uuid;
use crate::auth::ssh::schema::{AuthPayload, SSHAuthToken, SSH};
use crate::file::expand_tilde;

pub(crate) mod schema;

impl SSHAuthToken {
  pub fn new() -> anyhow::Result<Self> {
    let private_key_data = fs::read_to_string(SSH::get_default_ssh_key_path()?)?;

    let private_key = PrivateKey::from_openssh(&private_key_data)?;
    let payload = AuthPayload {
      timestamp: Utc::now().to_rfc3339(),
      nonce: Uuid::new_v4().to_string(),
    };
    let payload_json = serde_json::to_string(&payload)?;
    let signature = private_key.sign("dosei-ssh", HashAlg::default(), payload_json.as_bytes())
      .context("Failed to sign payload")?;

    // print!("{}", private_key.public_key().verify("dosei-ssh", payload_json.as_bytes(), &signature).is_ok());


    Ok(Self {
      payload: payload_json,
      signature: general_purpose::STANDARD.encode(signature.signature_bytes()),
    })
  }
}

impl SSH {
  pub fn execute_command(session: &Session, command: &str) -> anyhow::Result<(i32, String)> {
    let mut channel = session
      .channel_session()
      .context("Failed to create channel session")?;

    channel
      .exec(command)
      .context(format!("Failed to execute command: {}", command))?;

    let mut output = String::new();
    channel
      .read_to_string(&mut output)
      .context("Failed to read command output")?;

    channel.wait_close().context("Failed to close channel")?;

    let exit_status = channel.exit_status().context("Failed to get exit status")?;

    Ok((exit_status, output))
  }

  pub fn check_file_exists(session: &Session, path: &str) -> anyhow::Result<bool> {
    let (exit_code, _) = Self::execute_command(session, &format!("[ -f {} ] && echo 'exists'", path))?;
    Ok(exit_code == 0)
  }

  pub fn get_default_ssh_key_path() -> anyhow::Result<PathBuf> {

    let ed25519_path = expand_tilde("~/.ssh/id_ed25519");
    if let Ok(_) = fs::read_to_string(&ed25519_path) {
      return Ok(ed25519_path);
    }

    let rsa_path = expand_tilde("~/.ssh/id_rsa");
    if let Ok(_) = fs::read_to_string(&rsa_path) {
      return Ok(rsa_path);
    }
    Err(anyhow!("Could not find SSH keys. Tried ~/.ssh/id_ed25519 and ~/.ssh/id_rsa"))
  }
  pub fn get_public_key_from_private_key_path(private_key_path: &Path) -> anyhow::Result<(String, String)> {
    let private_key_data = fs::read_to_string(private_key_path)?;
    Ok(Self::get_public_key_from_private_key(private_key_data)?)
  }
  pub fn get_public_key_from_private_key(pem: impl AsRef<[u8]>) -> anyhow::Result<(String, String)> {
    let private_key = PrivateKey::from_openssh(pem)?;
    let public_key = private_key.public_key();
    Ok((public_key.to_openssh()?, public_key.comment().to_string()))
  }
}
