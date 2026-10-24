#!/usr/bin/env bash
set -euo pipefail

echo "========================================================"
echo " HypeHand Stellar / Soroban Dev Environment Verification"
echo "========================================================"

FAILED=0

# 1. Check Rust toolchain
echo -n "Checking rustc... "
if command -v rustc >/dev/null 2>&1; then
    RUST_VER=$(rustc --version)
    echo "OK ($RUST_VER)"
else
    echo "MISSING! Install Rust from https://rustup.rs"
    FAILED=1
fi

# 2. Check Cargo
echo -n "Checking cargo... "
if command -v cargo >/dev/null 2>&1; then
    CARGO_VER=$(cargo --version)
    echo "OK ($CARGO_VER)"
else
    echo "MISSING! Cargo is required."
    FAILED=1
fi

# 3. Check wasm32-unknown-unknown target
echo -n "Checking wasm32-unknown-unknown target... "
if rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "OK (Installed)"
else
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown || FAILED=1
fi

# 4. Check Stellar / Soroban CLI
echo -n "Checking stellar-cli / soroban-cli... "
if command -v stellar >/dev/null 2>&1; then
    STELLAR_VER=$(stellar --version 2>&1 | head -n 1)
    echo "OK ($STELLAR_VER)"
elif command -v soroban >/dev/null 2>&1; then
    SOROBAN_VER=$(soroban --version 2>&1 | head -n 1)
    echo "OK ($SOROBAN_VER)"
else
    echo "NOT FOUND in PATH."
    echo "  -> Note: For live network deployments, install via: 'cargo install --locked stellar-cli'"
    echo "  -> Alternatively, use Docker quickstart standalone container."
fi

# 5. Check Docker for local sandbox
echo -n "Checking docker daemon (for standalone sandbox)... "
if command -v docker >/dev/null 2>&1; then
    if docker info >/dev/null 2>&1; then
        echo "OK (Docker daemon running)"
    else
        echo "Docker installed but daemon is not reachable (start service or use remote RPC)"
    fi
else
    echo "NOT INSTALLED (optional: required only for local Docker quickstart node)"
fi

echo "========================================================"
if [ "$FAILED" -eq 0 ]; then
    echo "All core development prerequisites are satisfied! [READY]"
    exit 0
else
    echo "Environment setup failed. Please fix the missing dependencies above."
    exit 1
fi
