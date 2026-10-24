use std::fs;
use std::path::Path;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::error::KeystoreError;
use crate::secret::StellarSecretKey;

/// Serializable payload for encrypted Stellar keystore
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeystorePayload {
    pub version: u32,
    pub public_key: String,
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

/// AES-GCM-256 encrypted keystore for Stellar secret keys
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedKeystore {
    payload: KeystorePayload,
}

impl EncryptedKeystore {
    pub const CURRENT_VERSION: u32 = 1;

    /// Creates an `EncryptedKeystore` from an existing raw payload
    pub fn from_payload(payload: KeystorePayload) -> Self {
        Self { payload }
    }

    /// Borrows the internal payload
    pub fn payload(&self) -> &KeystorePayload {
        &self.payload
    }

    /// Returns the public key associated with the keystore
    pub fn public_key(&self) -> &str {
        &self.payload.public_key
    }

    /// Encrypts a `StellarSecretKey` with a password using Argon2id and AES-256-GCM
    pub fn encrypt(secret_key: &StellarSecretKey, password: &str) -> Result<Self, KeystoreError> {
        let mut salt = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt);

        let mut derived_key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut derived_key)
            .map_err(|e| KeystoreError::KdfError(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&derived_key)
            .map_err(|e| KeystoreError::KdfError(e.to_string()))?;

        // Zeroize key material immediately after cipher initialization
        derived_key.zeroize();

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, secret_key.expose_secret().as_bytes())
            .map_err(|_| KeystoreError::DecryptionFailed)?;

        let payload = KeystorePayload {
            version: Self::CURRENT_VERSION,
            public_key: secret_key.public_key().to_string(),
            salt: to_hex(&salt),
            nonce: to_hex(&nonce_bytes),
            ciphertext: to_hex(&ciphertext),
        };

        Ok(Self { payload })
    }

    /// Decrypts the keystore using the supplied password
    pub fn decrypt(&self, password: &str) -> Result<StellarSecretKey, KeystoreError> {
        let salt = from_hex(&self.payload.salt)?;
        let nonce_bytes = from_hex(&self.payload.nonce)?;
        let ciphertext = from_hex(&self.payload.ciphertext)?;

        if nonce_bytes.len() != 12 {
            return Err(KeystoreError::HexError("Invalid nonce length (must be 12 bytes)".to_string()));
        }

        let mut derived_key = [0u8; 32];
        Argon2::default()
            .hash_password_into(password.as_bytes(), &salt, &mut derived_key)
            .map_err(|e| KeystoreError::KdfError(e.to_string()))?;

        let cipher = Aes256Gcm::new_from_slice(&derived_key)
            .map_err(|e| KeystoreError::KdfError(e.to_string()))?;

        derived_key.zeroize();

        let nonce = Nonce::from_slice(&nonce_bytes);
        let mut decrypted_bytes = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| KeystoreError::DecryptionFailed)?;

        let secret_str = String::from_utf8(decrypted_bytes.clone())
            .map_err(|e| KeystoreError::HexError(e.to_string()))?;

        decrypted_bytes.zeroize();

        let secret_key = StellarSecretKey::from_str(&secret_str)?;

        // Sanity check public key matches
        if secret_key.public_key() != self.payload.public_key {
            return Err(KeystoreError::DecryptionFailed);
        }

        Ok(secret_key)
    }

    /// Saves the encrypted keystore JSON to a file with safe file permissions
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), KeystoreError> {
        let json_bytes = serde_json::to_vec_pretty(&self.payload)?;
        fs::write(path.as_ref(), json_bytes)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            let _ = fs::set_permissions(path.as_ref(), permissions);
        }

        Ok(())
    }

    /// Loads the encrypted keystore from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, KeystoreError> {
        let content = fs::read_to_string(path.as_ref())?;
        let payload: KeystorePayload = serde_json::from_str(&content)?;
        Ok(Self { payload })
    }

    /// Loads from file and decrypts in a single call
    pub fn load_and_decrypt<P: AsRef<Path>>(path: P, password: &str) -> Result<StellarSecretKey, KeystoreError> {
        let keystore = Self::load_from_file(path)?;
        keystore.decrypt(password)
    }
}

fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

fn from_hex(s: &str) -> Result<Vec<u8>, KeystoreError> {
    if !s.len().is_multiple_of(2) {
        return Err(KeystoreError::HexError("Hex string must have even length".to_string()));
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    for i in (0..s.len()).step_by(2) {
        let byte = u8::from_str_radix(&s[i..i + 2], 16)
            .map_err(|e| KeystoreError::HexError(format!("Invalid hex character at {}: {}", i, e)))?;
        bytes.push(byte);
    }
    Ok(bytes)
}
