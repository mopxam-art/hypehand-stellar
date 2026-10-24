/// Errors related to Stellar secret key operations and validation
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum KeyError {
    #[error("Stellar secret key must start with 'S', found '{0}'")]
    InvalidPrefix(char),

    #[error("Stellar secret key length must be 56 characters, found {0}")]
    InvalidLength(usize),

    #[error("Invalid character '{0}' in Stellar secret key (expected RFC 4648 Base32: A-Z, 2-7)")]
    InvalidBase32Char(char),

    #[error("Stellar secret key checksum or decoding failed: {0}")]
    InvalidChecksum(String),

    #[error("Environment variable '{0}' error: {1}")]
    EnvError(String, String),
}

/// Errors related to encrypted keystore operations
#[derive(Debug, thiserror::Error)]
pub enum KeystoreError {
    #[error("Key error: {0}")]
    Key(#[from] KeyError),

    #[error("Key derivation failed: {0}")]
    KdfError(String),

    #[error("Decryption failed (invalid password or corrupted keystore data)")]
    DecryptionFailed,

    #[error("Hex encoding/decoding failed: {0}")]
    HexError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),
}

impl PartialEq for KeystoreError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (KeystoreError::Key(a), KeystoreError::Key(b)) => a == b,
            (KeystoreError::KdfError(a), KeystoreError::KdfError(b)) => a == b,
            (KeystoreError::DecryptionFailed, KeystoreError::DecryptionFailed) => true,
            (KeystoreError::HexError(a), KeystoreError::HexError(b)) => a == b,
            (KeystoreError::Io(a), KeystoreError::Io(b)) => a.kind() == b.kind(),
            (KeystoreError::Json(a), KeystoreError::Json(b)) => a.to_string() == b.to_string(),
            _ => false,
        }
    }
}
