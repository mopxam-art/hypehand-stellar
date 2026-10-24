.PHONY: all build test build-contract optimize deploy-local setup-env clean fmt lint help

SHELL := /bin/bash

all: build test

help:
	@echo "HypeHand Stellar / Soroban Development Makefile"
	@echo "------------------------------------------------"
	@echo "make setup-env       - Verify & configure local dev dependencies"
	@echo "make build           - Build the entire workspace"
	@echo "make test            - Run all unit and integration tests"
	@echo "make build-contract  - Compile gifting contract to wasm32"
	@echo "make optimize        - Optimize compiled WASM binary"
	@echo "make deploy-local    - Deploy contract to local standalone sandbox"
	@echo "make fmt             - Format source code with rustfmt"
	@echo "make lint            - Run clippy linter with strict flags"
	@echo "make clean           - Remove build and deployment artifacts"

build:
	@echo "==> Building workspace..."
	cargo build --workspace

test:
	@echo "==> Running contract tests..."
	cargo test --workspace

build-contract:
	@echo "==> Compiling gifting contract to wasm32-unknown-unknown..."
	cargo build --package gifting-contract --target wasm32-unknown-unknown --release
	@ls -lh target/wasm32-unknown-unknown/release/gifting_contract.wasm

optimize: build-contract
	@echo "==> Optimizing contract WASM..."
	@if command -v stellar &> /dev/null; then \
		stellar contract optimize --wasm target/wasm32-unknown-unknown/release/gifting_contract.wasm; \
	elif command -v soroban &> /dev/null; then \
		soroban contract optimize --wasm target/wasm32-unknown-unknown/release/gifting_contract.wasm; \
	else \
		echo "Notice: stellar-cli not in PATH. Contract is already built with opt-level='z' and LTO."; \
	fi

deploy-local: build-contract
	@echo "==> Deploying contract to local sandbox..."
	bash scripts/deploy_local_contract.sh

setup-env:
	@echo "==> Checking development prerequisites..."
	bash scripts/setup_env.sh

fmt:
	@echo "==> Formatting code..."
	cargo fmt --all

lint:
	@echo "==> Running clippy..."
	cargo clippy --workspace --all-targets -- -D warnings

clean:
	@echo "==> Cleaning build artifacts..."
	cargo clean
	rm -rf .deployments
