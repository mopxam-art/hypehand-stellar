# Soroban Core Dependencies & Version Architecture

This document provides a comprehensive technical overview of the dependency architecture, version pinning rationale, and compatibility matrix for the **HypeHand** Soroban smart contracts.

---

## 1. Executive Summary & Version Matrix

The HypeHand smart contract workspace standardizes on **Soroban SDK v21.7.7** targeting **Stellar Protocol 21**.

| Component | Target Version | Rationale & Constraint |
|---|---|---|
| **`soroban-sdk`** | `21.7.7` | Stable release for Protocol 21 on Mainnet and Testnet |
| **`soroban-env-host`** | `21.2.1` | Host environment implementation embedded in test runner |
| **`soroban-env-common`** | `21.2.1` | Core Val/RawVal conversion and host function ABI |
| **`ed25519-dalek`** | `=2.1.1` | **Strictly pinned**: prevents breaking trait collision with `rand_core` |
| **Target Architecture** | `wasm32-unknown-unknown` | Deterministic WebAssembly compilation |
| **Rust Edition** | `2021` | Compatible with Rust 1.78+ and modern toolchains |

---

## 2. Breaking Conflict Resolution: `ed25519-dalek` v3 vs `soroban-env-host`

### The Problem
During automated workspace dependency resolution, Cargo may attempt to pull `ed25519-dalek` v3.0.0 due to semver range resolution. However, `ed25519-dalek` v3.0.0 introduced breaking changes in its cryptographic random number generator trait bounds (`CryptoRng` from `rand_core` 0.10).

When `soroban-env-host v21.2.1` compiles its mock authentication and test PRNG subsystem (`with_test_prng`), it calls `SigningKey::generate(chacha)`. Under `ed25519-dalek` v3, this causes a fatal compiler error:
```text
error[E0277]: the trait bound `ChaCha20Rng: ed25519_dalek::rand_core::CryptoRng` is not satisfied
  --> soroban-env-host-21.2.1/src/builtin_contracts/testutils.rs:26:58
   |
26 |     host.with_test_prng(|chacha| Ok(SigningKey::generate(chacha)))
   |                                     ^^^^^^^^^^^^^^^^^^^^ required by this bound
```

### The Solution
We enforce an exact pinned version constraint in `Cargo.toml` and lock it via `Cargo.lock`:
```toml
[workspace.dependencies]
soroban-sdk = { version = "21.7.7", default-features = false }
ed25519-dalek = "=2.1.1"
```
And enforce the lockfile resolution:
```bash
cargo update -p ed25519-dalek@3.0.0 --precise 2.1.1
```
This guarantees deterministic, zero-failure compilation across all local environments and CI pipelines.

---

## 3. Feature Flags Management

To ensure minimum WASM footprint and maintain strict `no_std` compliance for on-chain execution, feature flags are cleanly decoupled:

### Contract `[features]` Configuration:
```toml
[features]
default = []
alloc = ["soroban-sdk/alloc"]
testutils = ["soroban-sdk/testutils"]

[dependencies]
soroban-sdk = { workspace = true, features = ["alloc"] }

[dev-dependencies]
soroban-sdk = { workspace = true, features = ["testutils"] }
ed25519-dalek = { workspace = true }
```

- **`alloc`**: Enables heap allocations required for dynamic types (`Vec`, `Map`, `String`, `Bytes`) inside the contract.
- **`testutils`**: Enables local test environment simulation (`Env::default()`, `Address::generate()`, `mock_all_auths()`). **Never enabled in release WASM builds.**

---

## 4. Release Profile Optimization Specs

Soroban contracts have strict CPU instruction and byte-size limits on Stellar. The root `Cargo.toml` specifies aggressive size optimizations:

```toml
[profile.release]
opt-level = "z"         # Optimize aggressively for smallest binary size
overflow-checks = true  # Guard against arithmetic overflow even in release
debug = 0               # Strip all debug info
strip = "symbols"       # Strip symbol tables
panic = "abort"         # Abort on panic to remove unwind landing pads
codegen-units = 1       # Single codegen unit allows maximum LTO inlining
lto = true              # Full Link-Time Optimization
```

Result: The compiled `gifting.wasm` binary occupies only **3.4 KB**, well below network deployment limits.

---

## 5. Dependency Upgrade Runbook

When Stellar Protocol 22 is finalized on Mainnet, follow this checklist to safely upgrade:

1. **Check Compatibility**: Verify the updated version of `soroban-sdk` (e.g., `22.x`) on [Stellar Developer Docs](https://developers.stellar.org).
2. **Update Root Manifest**:
   ```toml
   [workspace.dependencies]
   soroban-sdk = { version = "22.0.0", default-features = false }
   ```
3. **Audit Transitive Dependencies**:
   ```bash
   cargo check --dry-run
   cargo tree -p soroban-env-host
   ```
4. **Run Test Matrix**:
   ```bash
   cargo test -p gifting
   cargo build -p gifting --release --target wasm32-unknown-unknown
   ```
5. **Verify Lockfile**: Commit the updated `Cargo.lock` alongside manifest changes.
