# Local Soroban Development Environment Guide

Welcome to the **HypeHand** Soroban smart contract development environment. This guide explains how to set up, build, test, and deploy smart contracts locally on the Stellar network.

---

## 1. Prerequisites

Before developing or deploying contracts, ensure your machine has the following tools installed:

1. **Rust & Cargo** (v1.78+ or latest stable):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **WebAssembly Target (`wasm32-unknown-unknown`)**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Stellar CLI** (optional for local mock / required for live deployments):
   ```bash
   cargo install --locked stellar-cli
   ```

4. **Docker** (for running local standalone Stellar/Soroban node):
   - Required only if running the full local quickstart sandbox.

---

## 2. Automated Environment Verification

To verify that all required dependencies are installed, run:

```bash
make setup-env
# or directly:
bash scripts/setup_env.sh
```

---

## 3. Makefile Targets

We provide a streamlined `Makefile` automating common development lifecycle workflows:

| Target | Description |
|---|---|
| `make build` | Compiles the entire Rust workspace |
| `make test` | Runs all contract unit tests with full snapshot assertions |
| `make build-contract` | Compiles `gifting-contract` to `wasm32-unknown-unknown` release binary |
| `make optimize` | Optimizes WASM size using `stellar-cli` / `wasm-opt` |
| `make deploy-local` | Deploys contract to local sandbox and creates deployment JSON |
| `make setup-env` | Checks toolchain prerequisites and missing targets |
| `make fmt` | Formats all Rust code according to workspace `rustfmt.toml` |
| `make lint` | Runs `clippy` with strict workspace warning flags |
| `make clean` | Removes `target/` and `.deployments/` directories |

---

## 4. Running a Local Standalone Sandbox

To test against an active Soroban RPC without using public testnets, launch the official Stellar quickstart container:

```bash
docker run --rm -it \
  -p 8000:8000 \
  --name stellar-standalone \
  stellar/quickstart:testing \
  --standalone \
  --enable-rpc
```

- **RPC Endpoint:** `http://localhost:8000/soroban/rpc`
- **Network Passphrase:** `Standalone Network ; February 2017`
- **Friendbot (faucet):** `http://localhost:8000/friendbot`

---

## 5. Building & Deploying the Contract

### Step 1: Build the Contract WASM
```bash
make build-contract
```
The output binary will be located at:
`target/wasm32-unknown-unknown/release/gifting_contract.wasm`

### Step 2: Deploy to Local Sandbox
```bash
make deploy-local
```
Or customize options via CLI flags:
```bash
bash scripts/deploy_local_contract.sh \
  --network standalone \
  --rpc-url http://localhost:8000/soroban/rpc \
  --source alice \
  --output .deployments/local_deployment.json
```

A deployment metadata file is generated at `.deployments/local_deployment.json`:
```json
{
  "contract_name": "gifting_contract",
  "contract_id": "CA3D5KRYMCMCZKPO7SJ2VQA7EKNHQQLCPF5K6E7X7A",
  "network": "standalone",
  "rpc_url": "http://localhost:8000/soroban/rpc",
  "wasm_sha256": "...",
  "status": "ready"
}
```

---

## 6. Contract Invocations (CLI Examples)

Once deployed, interact with the contract using the `stellar` CLI or HypeHand CLI tool:

### 1. Create an Account
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network standalone \
  -- \
  create_account \
  --user <USER_PUBLIC_KEY> \
  --initial_deposit 1000000000
```

### 2. Check Balance
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network standalone \
  -- \
  get_balance \
  --user <USER_PUBLIC_KEY>
```

### 3. Send a Gift (Deposit to Recipient)
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source alice \
  --network standalone \
  -- \
  deposit \
  --from <SENDER_PUBLIC_KEY> \
  --to <RECIPIENT_PUBLIC_KEY> \
  --amount 500000000
```

---

## 7. Troubleshooting

- **Error: `target wasm32-unknown-unknown not found`**  
  Run `rustup target add wasm32-unknown-unknown`.
- **WASM size too large:**  
  Make sure release profile includes `opt-level = "z"`, `lto = true`, and `strip = "symbols"`.
- **Authorization failure (`HostError: Error(Auth, InvalidAction)`):**  
  Ensure the caller matches the user whose balance is being manipulated; Soroban enforces cryptographic authorization via `user.require_auth()`.
