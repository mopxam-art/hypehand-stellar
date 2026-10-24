use ed25519_dalek::SigningKey;
use rand::RngCore;
use secrecy::{ExposeSecret, SecretString};
use std::fmt;
use stellar_strkey::ed25519::{PrivateKey, PublicKey};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::KeyError;

/// Raw 32-byte secret seed with automatic zeroization on drop
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ZeroizedSeed(pub [u8; 32]);

impl fmt::Debug for ZeroizedSeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ZeroizedSeed([REDACTED])")
    }
}

/// Secure container for a Stellar secret key (SEP-23 ed25519 secret seed)
///
/// Memory is protected using `secrecy::SecretString` and `zeroize::ZeroizeOnDrop`
/// to prevent secret material from persisting in RAM dumps or appearing in logs.
pub struct StellarSecretKey {
    secret: SecretString,
    seed: ZeroizedSeed,
    public_address: String,
}

impl fmt::Debug for StellarSecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StellarSecretKey")
            .field("public_address", &self.public_address)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

impl std::str::FromStr for StellarSecretKey {
    type Err = KeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        validate_stellar_secret_key(trimmed)?;

        let priv_strkey = PrivateKey::from_string(trimmed)
            .map_err(|e| KeyError::InvalidChecksum(format!("{:?}", e)))?;

        let seed = ZeroizedSeed(priv_strkey.0);
        let signing_key = SigningKey::from_bytes(&seed.0);
        let verifying_key = signing_key.verifying_key();
        let pub_strkey = PublicKey(verifying_key.to_bytes());
        let public_address = pub_strkey.to_string().to_string();

        Ok(Self { secret: SecretString::from(trimmed.to_string()), seed, public_address })
    }
}

impl StellarSecretKey {
    /// Validates and constructs a `StellarSecretKey` from a string slice
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(key_str: &str) -> Result<Self, KeyError> {
        <Self as std::str::FromStr>::from_str(key_str)
    }

    /// Constructs a `StellarSecretKey` from a protected `SecretString`
    pub fn from_secret_string(secret: SecretString) -> Result<Self, KeyError> {
        Self::from_str(secret.expose_secret())
    }

    /// Loads a `StellarSecretKey` from an environment variable (e.g. `STELLAR_SECRET_KEY`)
    pub fn from_env(var_name: &str) -> Result<Self, KeyError> {
        let val = std::env::var(var_name)
            .map_err(|e| KeyError::EnvError(var_name.to_string(), e.to_string()))?;
        Self::from_str(&val)
    }

    /// Constructs a `StellarSecretKey` directly from raw 32-byte entropy
    pub fn from_raw_seed(raw_seed: [u8; 32]) -> Self {
        let priv_strkey = PrivateKey(raw_seed);
        let unredacted = priv_strkey.as_unredacted().to_string();
        let signing_key = SigningKey::from_bytes(&raw_seed);
        let verifying_key = signing_key.verifying_key();
        let pub_strkey = PublicKey(verifying_key.to_bytes());
        let public_address = pub_strkey.to_string().to_string();

        Self {
            secret: SecretString::from(unredacted.to_string()),
            seed: ZeroizedSeed(raw_seed),
            public_address,
        }
    }

    /// Generates a new cryptographically secure random `StellarSecretKey`
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut seed);
        Self::from_raw_seed(seed)
    }

    /// Safely returns the Stellar public address (`G...`) without exposing the secret
    pub fn public_key(&self) -> &str {
        &self.public_address
    }

    /// Explicitly reveals the secret key string for signing operations
    pub fn expose_secret(&self) -> &str {
        self.secret.expose_secret()
    }

    /// Borrows the internal zeroize-protected 32-byte seed
    pub fn raw_seed(&self) -> &[u8; 32] {
        &self.seed.0
    }
}

/// Validates the format and checksum of a Stellar Secret Key (`S[A-Z2-7]{55}`)
pub fn validate_stellar_secret_key(s: &str) -> Result<(), KeyError> {
    if s.len() != 56 {
        return Err(KeyError::InvalidLength(s.len()));
    }

    let first_char = s.chars().next().unwrap_or('\0');
    if first_char != 'S' {
        return Err(KeyError::InvalidPrefix(first_char));
    }

    for c in s.chars() {
        if !matches!(c, 'A'..='Z' | '2'..='7') {
            return Err(KeyError::InvalidBase32Char(c));
        }
    }

    PrivateKey::from_string(s).map_err(|e| KeyError::InvalidChecksum(format!("{:?}", e)))?;

    Ok(())
}
