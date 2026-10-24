#!/usr/bin/env bash
set -euo pipefail

echo "=========================================="
echo " HypeHand Code Style & Linter Check       "
echo "=========================================="

echo "==> 1. Checking code formatting (rustfmt)..."
cargo fmt --all -- --check

echo "==> 2. Checking compiler lints (clippy)..."
cargo clippy --workspace --all-targets -- -D warnings

echo "=========================================="
echo " All style and lint checks passed! [OK]   "
echo "=========================================="
