use anyhow::{anyhow, Context};
use base64::engine::general_purpose;
use base64::Engine;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::auth::ssh::schema::{SSHBearerPayload, SSH};
use crate::file::expand_tilde;
use ssh2::Session;
use ssh_key::rand_core::OsRng;
use ssh_key::{Algorithm, HashAlg, LineEnding, PrivateKey, PublicKey, Signature, SshSig};
use uuid::Uuid;

pub(crate) mod schema;

const DOSEI_SSH_NAMESPACE: &str = "dosei-ssh";

impl SSHBearerPayload {
  pub fn new() -> anyhow::Result<Self> {
    let private_key_data = fs::read_to_string(SSH::get_default_ssh_key_path()?)?;
    let private_key = PrivateKey::from_openssh(&private_key_data)?;

    let nonce = Uuid::new_v4().to_string();
    let signature = private_key
      .sign(DOSEI_SSH_NAMESPACE, HashAlg::default(), nonce.as_bytes())
      .context("Failed to sign payload")?;

    Ok(Self {
      namespace: DOSEI_SSH_NAMESPACE.to_string(),
      nonce,
      signature: signature.signature_bytes().to_vec(),
    })
  }
  pub fn to_base64(&self) -> anyhow::Result<String> {
    let json = serde_json::to_string(self)?;
    Ok(general_purpose::STANDARD.encode(json.as_bytes()))
  }
  pub fn from_base64(base64: &str) -> anyhow::Result<Self> {
    let bytes = general_purpose::STANDARD.decode(base64)?;
    let payload: Self = serde_json::from_slice(&bytes)?;
    Ok(payload)
  }

  pub fn verify(&self, public_key: &PublicKey) -> bool {
    // Create a Signature object from the raw bytes
    let signature = match Signature::new(public_key.algorithm(), self.signature.clone()) {
      Ok(sig) => sig,
      Err(_) => return false, // Invalid signature format
    };

    // Create the SSH signature with all required metadata
    let ssh_sig = match SshSig::new(
      public_key.key_data().clone(),
      &self.namespace,
      HashAlg::default(),
      signature,
    ) {
      Ok(sig) => sig,
      Err(_) => return false, // Failed to create SSH signature
    };

    // Verify the complete signature
    public_key
      .verify(&self.namespace, self.nonce.as_bytes(), &ssh_sig)
      .is_ok()
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
    let (exit_code, _) =
      Self::execute_command(session, &format!("[ -f {} ] && echo 'exists'", path))?;
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
    Err(anyhow!(
      "Could not find SSH keys. Tried ~/.ssh/id_ed25519 and ~/.ssh/id_rsa"
    ))
  }
  pub fn get_public_key_from_private_key_path(
    private_key_path: &Path,
  ) -> anyhow::Result<(String, String)> {
    let private_key_data = fs::read_to_string(private_key_path)?;
    Ok(Self::get_public_key_from_private_key(private_key_data)?)
  }
  pub fn get_public_key_from_private_key(
    pem: impl AsRef<[u8]>,
  ) -> anyhow::Result<(String, String)> {
    let private_key = PrivateKey::from_openssh(pem)?;
    let public_key = private_key.public_key();
    Ok((public_key.to_openssh()?, public_key.comment().to_string()))
  }

  pub fn generate_ed25519_key() -> anyhow::Result<String> {
    let mut rng = OsRng::default();
    let current_dir = std::env::current_dir()?;
    let dosei_dir = current_dir.join(".dosei");
    if !dosei_dir.exists() {
      fs::create_dir_all(&dosei_dir).context("Failed to create .dosei directory")?;
    }

    let private_key_path = dosei_dir.join("dosei_ed25519");
    let public_key_path = dosei_dir.join("dosei_ed25519.pub");

    if private_key_path.exists() && public_key_path.exists() {
      return Ok(fs::read_to_string(public_key_path)?.trim().to_string());
    }

    let keypair = PrivateKey::random(&mut rng, Algorithm::Ed25519)?;
    let key = PrivateKey::from(keypair);

    let private_key = key.to_openssh(LineEnding::default())?;
    let public_key = key.public_key().to_openssh()?;

    fs::write(&private_key_path, private_key.as_bytes()).context(format!(
      "Failed to write key to {}",
      private_key_path.display()
    ))?;

    fs::write(&public_key_path, public_key.as_bytes()).context(format!(
      "Failed to write key to {}",
      public_key_path.display()
    ))?;

    // Set appropriate permissions (read/write for owner only)
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let metadata = fs::metadata(&private_key_path)?;
      let mut permissions = metadata.permissions();
      permissions.set_mode(0o600); // Read/write for owner only
      fs::set_permissions(&private_key_path, permissions)?;
    }

    Ok(public_key)
  }
}

#[cfg(test)]
mod tests {
  use crate::auth::ssh::schema::{SSHBearerPayload, SSH};
  use ssh_key::PrivateKey;
  use std::fs;

  #[test]
  fn verify_ssh_key_verify() {
    let private_key_data = fs::read_to_string(SSH::get_default_ssh_key_path().unwrap()).unwrap();
    let private_key = PrivateKey::from_openssh(&private_key_data).unwrap();

    let payload = SSHBearerPayload::new().unwrap();

    assert_eq!(true, payload.verify(private_key.public_key()));
  }

  #[test]
  fn generate_dosei_key() {
    SSH::generate_ed25519_key().unwrap();
  }
}
