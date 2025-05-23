use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};
use ssh_key::{HashAlg, PublicKey, Signature, SshSig};

pub const DOSEI_SSH_NAMESPACE: &str = "dosei-ssh";

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SSHBearerPayload {
  pub namespace: String,
  pub nonce: String,
  pub key_fingerprint: String,
  pub signature: Vec<u8>,
}

impl SSHBearerPayload {
  pub fn to_base64(&self) -> anyhow::Result<String> {
    let json = serde_json::to_string(self)?;
    Ok(general_purpose::STANDARD.encode(json.as_bytes()))
  }
  pub fn from_base64(base64: &str) -> anyhow::Result<Self> {
    let bytes = general_purpose::STANDARD.decode(base64)?;
    let payload: Self = serde_json::from_slice(&bytes)?;
    Ok(payload)
  }

  pub fn fingerprint_from_public_key(public_key: &str) -> anyhow::Result<String> {
    let key = PublicKey::from_openssh(public_key)?;
    let fingerprint = key.fingerprint(HashAlg::default());
    Ok(fingerprint.to_string())
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
  pub fn verify_from_string(&self, public_key: String) -> bool {
    self.verify(&PublicKey::from_openssh(&public_key).unwrap())
  }
}
