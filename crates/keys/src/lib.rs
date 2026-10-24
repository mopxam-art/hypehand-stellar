//! Secure Secret Management and Encrypted Keystore for Stellar Keys
//!
//! Provides zeroize-protected memory buffers, secrecy wrappers,
//! AES-256-GCM + Argon2id keystore encryption, and Stellar Strkey validation.

pub mod error;
pub mod keystore;
pub mod secret;

pub use error::{KeyError, KeystoreError};
pub use keystore::{EncryptedKeystore, KeystorePayload};
pub use secret::{validate_stellar_secret_key, StellarSecretKey, ZeroizedSeed};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use zeroize::Zeroize;

    const VALID_SECRET: &str = "SAVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCVLG5";

    #[test]
    fn test_valid_secret_key_parsing_and_derivation() {
        let key = StellarSecretKey::from_str(VALID_SECRET).expect("Valid secret should parse");

        // Secret should match unredacted value
        assert_eq!(key.expose_secret(), VALID_SECRET);

        // Public key derivation should yield valid Stellar G... address
        let pub_key = key.public_key();
        assert!(pub_key.starts_with('G'));
        assert_eq!(pub_key.len(), 56);
        assert_eq!(pub_key, "GAMX62ZD4FWIKMWGVPEDR6WNL2TYTPQMO2ZJEAZUAON7VCZ5G2GWDF7W");
    }

    #[test]
    fn test_random_key_generation() {
        let key1 = StellarSecretKey::generate();
        let key2 = StellarSecretKey::generate();

        assert!(key1.public_key().starts_with('G'));
        assert!(key2.public_key().starts_with('G'));
        assert_ne!(key1.public_key(), key2.public_key());
        assert_ne!(key1.expose_secret(), key2.expose_secret());
    }

    #[test]
    fn test_invalid_keys_validation() {
        // Invalid length (55 chars)
        let short_key = "SAVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCVLG";
        assert!(matches!(
            validate_stellar_secret_key(short_key),
            Err(KeyError::InvalidLength(55))
        ));

        // Invalid prefix (G instead of S)
        let invalid_prefix = "GAVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCVLG5";
        assert!(matches!(
            validate_stellar_secret_key(invalid_prefix),
            Err(KeyError::InvalidPrefix('G'))
        ));

        // Invalid character ('8' and '9' are not in RFC4648 base32)
        let invalid_char = "SAVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCVLG8";
        assert!(matches!(
            validate_stellar_secret_key(invalid_char),
            Err(KeyError::InvalidBase32Char('8'))
        ));

        // Invalid checksum (last character altered to a valid base32 char)
        let bad_checksum = "SAVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCUKRKFIVCVLG4";
        assert!(matches!(
            validate_stellar_secret_key(bad_checksum),
            Err(KeyError::InvalidChecksum(_))
        ));
    }

    #[test]
    fn test_memory_protection_and_debug_redaction() {
        let key = StellarSecretKey::from_str(VALID_SECRET).unwrap();
        let debug_output = format!("{:?}", key);

        // Debug output must NOT contain the secret string
        assert!(!debug_output.contains(VALID_SECRET));
        assert!(debug_output.contains("[REDACTED]"));
        assert!(debug_output.contains("StellarSecretKey"));

        // Raw seed debug must also redact
        let seed = ZeroizedSeed([123u8; 32]);
        let seed_debug = format!("{:?}", seed);
        assert!(!seed_debug.contains("123"));
        assert!(seed_debug.contains("[REDACTED]"));
    }

    #[test]
    fn test_zeroize_clears_buffer() {
        let mut buffer = [0xAAu8; 32];
        assert_eq!(buffer[0], 0xAA);
        buffer.zeroize();
        assert_eq!(buffer, [0u8; 32]);

        let mut zeroized_seed = ZeroizedSeed([0x77u8; 32]);
        assert_eq!(zeroized_seed.0[0], 0x77);
        zeroized_seed.zeroize();
        assert_eq!(zeroized_seed.0, [0u8; 32]);
    }

    #[test]
    fn test_keystore_encryption_and_decryption_roundtrip() {
        let key = StellarSecretKey::from_str(VALID_SECRET).unwrap();
        let password = "correct_horse_battery_staple";

        // Encrypt key
        let keystore = EncryptedKeystore::encrypt(&key, password).expect("Encryption failed");
        assert_eq!(keystore.public_key(), key.public_key());
        assert!(!keystore.payload().ciphertext.is_empty());
        assert!(!keystore.payload().salt.is_empty());

        // Decrypt with correct password
        let decrypted = keystore.decrypt(password).expect("Decryption should succeed");
        assert_eq!(decrypted.expose_secret(), VALID_SECRET);
        assert_eq!(decrypted.public_key(), key.public_key());

        // Decrypt with incorrect password should fail
        let wrong_password_result = keystore.decrypt("wrong_password");
        assert!(matches!(
            wrong_password_result,
            Err(KeystoreError::DecryptionFailed)
        ));
    }

    #[test]
    fn test_keystore_file_persistence() {
        let key = StellarSecretKey::from_str(VALID_SECRET).unwrap();
        let password = "keystore_file_secret_pass";

        let keystore = EncryptedKeystore::encrypt(&key, password).unwrap();
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Save to file
        keystore.save_to_file(path).expect("Failed to save keystore file");

        // Load and decrypt
        let loaded_key = EncryptedKeystore::load_and_decrypt(path, password)
            .expect("Failed to load and decrypt keystore file");
        assert_eq!(loaded_key.expose_secret(), VALID_SECRET);
        assert_eq!(loaded_key.public_key(), key.public_key());
    }

    #[test]
    fn test_load_from_env_var() {
        let env_var = "TEST_STELLAR_SECRET_KEY_ENV";
        std::env::set_var(env_var, VALID_SECRET);

        let key = StellarSecretKey::from_env(env_var).expect("Loading from env should succeed");
        assert_eq!(key.expose_secret(), VALID_SECRET);

        // Missing env var returns error
        assert!(matches!(
            StellarSecretKey::from_env("NON_EXISTENT_SECRET_ENV_VAR"),
            Err(KeyError::EnvError(_, _))
        ));

        std::env::remove_var(env_var);
    }
}
