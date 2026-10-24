# 🏛️ HypeHand Workspace Architecture

This document describes the Cargo workspace architecture and multi-crate monorepo design for the HypeHand Stellar platform.

---

## 📂 Monorepo Structure

```
hypehand-stellar/
├── Cargo.toml                    # Root workspace manifest & shared dependency definitions
├── Cargo.lock                    # Locked dependency versions
├── contracts/
│   └── gifting/                  # Soroban smart contract (no_std, wasm32-unknown-unknown)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs            # Contract entrypoints: init, version, get_admin, is_initialized
│           └── test.rs           # Soroban host test suite (5 tests)
├── crates/
│   ├── cli/                      # HypeHand CLI tool (hypehand-cli)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs           # Commands: deploy, invoke-gift, balance, status, convert (10 tests)
│   └── utils/                    # Core utilities library (hypehand-utils)
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs            # Address validation, stroops <-> XLM conversion, limits (8 tests)
├── backend/                      # NestJS backend service
├── frontend/                     # Next.js web application
└── docs/                         # Technical architecture documentation
    └── WORKSPACE_ARCHITECTURE.md # This guide
```

---

## ⚙️ Workspace Configuration

### 1. Root `Cargo.toml`
The root manifest coordinates all Rust crates and smart contracts:
- **`resolver = "2"`**: Modern feature resolution preventing dev-dependency feature bleed into production builds.
- **`[workspace.package]`**: Common metadata shared by all crates (`version = "0.1.0"`, `edition = "2021"`, dual MIT/Apache-2.0 license).
- **`[workspace.dependencies]`**: Central source of truth for version constraints:
  - `soroban-sdk = "21.7.7"` (Protocol 21 compatible)
  - `ed25519-dalek = "=2.1.1"` (pinned to prevent `rand_core` 0.9 trait collisions with `soroban-env-host`)
  - `clap = { version = "4.5", features = ["derive"] }`
  - `serde = { version = "1.0", features = ["derive"] }`
  - `tokio = { version = "1.38", features = ["full"] }`
  - `thiserror = "1.0"`

### 2. Binary & Contract Optimization Profiles
Release profiles are tuned specifically for Soroban WASM size constraints:
```toml
[profile.release]
opt-level = "z"        # Optimize strictly for minimal bytecode size
overflow-checks = true # Safety checks enabled in production
debug = 0              # Strip debug symbols
strip = "symbols"      # Minimize binary size
panic = "abort"        # No unwind overhead
codegen-units = 1      # Full LTO unit optimization
lto = true             # Link-time optimization
```

---

## 📦 Workspace Members

| Member | Package Name | Crate Type | Purpose |
|---|---|---|---|
| `contracts/gifting` | `gifting` | `cdylib`, `rlib` | On-chain gifting smart contract running in Soroban environment |
| `crates/cli` | `hypehand-cli` | `bin` | Management CLI for contract deployment, querying, and invoking gifts |
| `crates/utils` | `hypehand-utils` | `lib` | Shared utilities: address parsing, stroops conversions, timelocks |

---

## 🛠️ Verification & Build Commands

### 1. Workspace Verification
```bash
# Check all workspace members
cargo check --workspace

# Run all workspace unit and integration tests (23 tests)
cargo test --workspace

# Run strict clippy linter
cargo clippy --workspace -- -D warnings
```

### 2. Building On-Chain WASM Contract
```bash
cargo build -p gifting --release --target wasm32-unknown-unknown
```

### 3. Running CLI Utility
```bash
# Display CLI help
cargo run -p hypehand-cli -- --help

# Convert currency
cargo run -p hypehand-cli -- convert --stroops 50000000
cargo run -p hypehand-cli -- convert --xlm 5.0
```
