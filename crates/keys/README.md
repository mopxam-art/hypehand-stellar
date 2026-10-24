# hypehand-keys

Secure secret management, memory protection, and AES-256-GCM encrypted keystore for Stellar keys.

## Features

- Zeroize-protected memory buffers for secret seed material
- `secrecy::SecretString` wrapping to prevent log and dump leaks
- SEP-23 Strkey validation and safe Ed25519 public key derivation
- Argon2id password-based key derivation with AES-256-GCM file encryption
