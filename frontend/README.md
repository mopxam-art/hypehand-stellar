# 🎁 HypeHand Frontend — Web3 Gifting & Tipping Platform

Next.js 14 frontend application for **HypeHand**, a high-speed creator tipping and gifting platform built on **Stellar Soroban smart contracts**.

---

## 🚀 Features

- **Next.js 14 App Router:** Modern server and client components with clean routing architecture.
- **Tailwind CSS Styling:** Dark-mode optimized, responsive UI with sleek gradient accents.
- **Stellar Soroban Integration:** Designed to interact with Soroban RPC and contract endpoints for account management, direct micro-tips, and transaction receipts.
- **Key Application Routes:**
  - `/` — Platform landing page, highlighting feature value propositions, transaction finality, and network stats.
  - `/dashboard` — Creator dashboard showing verified status, real-time XLM balances, and received gifts activity log.
  - `/gift` — Interactive gifting form supporting presets, custom XLM amounts, personal messages, and onchain simulation.

---

## 🛠️ Monorepo Scripts

Run the frontend commands from either the repository root or the `frontend/` directory:

### From repository root:
```bash
# Start Next.js development server (http://localhost:3000)
npm run frontend:dev

# Build production bundle and verify TypeScript types
npm run frontend:build

# Run ESLint validation
npm run frontend:lint
```

### From `frontend/` directory:
```bash
npm run dev
npm run build
npm run lint
```

---

## ⚙️ Environment Configuration

Copy the example environment configuration:

```bash
cp .env.local.example .env.local
```

| Variable | Description | Default |
|---|---|---|
| `NEXT_PUBLIC_STELLAR_NETWORK` | Stellar target network (`testnet` / `mainnet`) | `testnet` |
| `NEXT_PUBLIC_STELLAR_RPC_URL` | Soroban RPC endpoint URL | `https://soroban-testnet.stellar.org` |
| `NEXT_PUBLIC_CONTRACT_ID` | Deployed HypeHand Soroban contract ID | `CAAA...` |
| `NEXT_PUBLIC_API_URL` | Backend NestJS service URL | `http://localhost:3001` |
