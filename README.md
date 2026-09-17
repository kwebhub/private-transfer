<!-- Markdownlint-disable MD013 -->
# zk-pool

> Private SOL transfers on Solana using Groth16 zero-knowledge proofs.

[![CI](https://github.com/kwebhub/private-transfer/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kwebhub/private-transfer/actions/workflows/ci.yml)
[![Security](https://github.com/kwebhub/private-transfer/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/kwebhub/private-transfer/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana)](https://solana.com)
[![Noir](https://img.shields.io/badge/Noir-1.0.0--rc.1-blue)](https://noir-lang.org)

[Русская версия](docs/ru/README.md) · [English](README.md)

---

**zk-pool** is a private pool for SOL transfers. A user deposits SOL into a shared
vault, receives a *deposit note* (two secrets), and can later withdraw SOL to
**any** address without revealing the link between deposit and withdrawal.

The project implements the full stack of a ZK protocol:

- **Noir** — ZK circuit with Poseidon2 and Merkle proof.
- **Sunspot** — Noir → Groth16 proof for Solana.
- **Anchor** — Solana program with an on-chain verifier.
- **Rust + axum** — backend with indexer and Merkle tree in Redis.
- **Vue 3 + Vite** — frontend with client-side witness generation.
- **Postgres + Redis** — storage and cache.
- **Prometheus + Grafana** — monitoring.

> ⚠️ **Status:** devnet, portfolio project. Do not use in mainnet without audit.

---

## Table of Contents

- [How It Works](#how-it-works)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Repository Structure](#repository-structure)
- [Tech Stack](#tech-stack)
- [Commands](#commands)
- [Monitoring](#monitoring)
- [Security](#security)
- [Documentation](#documentation)
- [License](#license)

---

## How It Works

### Deposit

1. Frontend generates two secrets: `nullifierSecret`, `secret`.
2. Computes `commitment = Poseidon2(nullifierSecret, secret, amount)`.
3. Requests the current Merkle root from the backend.
4. Sends a `deposit` tx to the program: SOL goes to the vault, the commitment becomes a leaf in the tree.
5. Stores the *deposit note* (secrets + commitment) in localStorage.

### Withdrawal

1. Frontend parses the note, requests a Merkle proof from the backend.
2. Generates a ZK witness (noir_js) — a proof that it knows the secrets
   corresponding to a leaf in the tree, **without revealing** them.
3. Backend proxies the witness to the prover (Sunspot), receives a Groth16 proof.
4. Frontend sends a `withdraw` tx: the program verifies the proof on-chain
   via the verifier program and transfers SOL from the vault.
5. A `NullifierRecord` PDA is created — repeated withdrawal with the same nullifier is impossible.

**What the blockchain sees:** commitment, nullifier_hash, root, amount.
**What is hidden:** the link between deposit and withdrawal.

More — [`docs/zk-explained.md`](docs/zk-explained.md).

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (Vue 3)                                           │
│  Deposit: commitment + tx                                   │
│  Withdraw: proof request + witness + tx                     │
└─────────────────────────────────────────────────────────────┘
              ↓ HTTP                    ↑ HTTP
┌─────────────────────────────────────────────────────────────┐
│  Backend (Rust + axum)                                      │
│  /api/commitments  /api/root  /api/proof  /api/withdraw     │
│  Indexer: listens to events → Postgres → Merkle tree        │
│  Rate limiting · Prometheus metrics                         │
└─────────────────────────────────────────────────────────────┘
       ↓                ↓                ↓
┌────────────┐   ┌────────────┐   ┌──────────────────┐
│ PostgreSQL │   │   Redis    │   │  Merkle (Node.js)│
│ commitments│   │ cache/tree │   │  Noir Poseidon2  │
│ nullifiers │   │ rate limit │   │  /hash /root     │
│ roots      │   │            │   │  /proof          │
└────────────┘   └────────────┘   └──────────────────┘
                                            ↑
┌───────────────────────────────────────────┴─────────────────┐
│  Solana (Anchor program `ptrans`)                           │
│  Pool PDA · Vault PDA · NullifierRecord PDA                 │
│  On-chain Groth16 verifier (Sunspot)                        │
└─────────────────────────────────────────────────────────────┘
```

More — [`docs/architecture.md`](docs/architecture.md).

---

## Quick Start

### Requirements

- Linux (Debian 12+ / Ubuntu 22.04+)
- Docker + Docker Compose
- Rust 1.89+ ([rustup](https://rustup.rs))
- Node.js 22+ ([nvm](https://github.com/nvm-sh/nvm))
- pnpm (`npm install -g pnpm`)
- tmux (`sudo apt install tmux`)
- Anchor 0.32.2
- Solana CLI
- Noir (`nargo`)

### Installation

```bash
git clone https://github.com/kwebhub/private-transfer.git
cd private-transfer

# 1. Start Docker environment (solana-dev, postgres, redis, prometheus, grafana)
docker compose up -d

# 2. Build and deploy the program to devnet
cd workspace/ptrans
anchor build
anchor deploy --provider.cluster devnet
cd ../..

# 3. Copy .env.example → .env (to the right places)
cp .env.example workspace/ptrans/.env
cp workspace/ptrans/.env.example workspace/ptrans/app/frontend/.env
# edit values for your wallet

# 4. Apply DB migrations
docker compose exec -T postgres psql -U ptrans -d ptrans < workspace/ptrans/app/backend/migrations/001_init.sql

# 5. Install frontend and merkle dependencies
cd workspace/ptrans/app/frontend && pnpm install && cd -
cd workspace/ptrans/app/merkle && pnpm install && cd -

# 6. Start all services with one command
./start-all.sh
```

After startup:

- Frontend: http://localhost:5173
- Backend API: http://localhost:4001
- Merkle: http://localhost:4003
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000 (admin / admin)

### Stop

```bash
./start-all.sh stop
docker compose down
```

---

## Repository Structure

```
private-transfer/
├── programs/ptrans/          ← Solana program (Anchor)
├── workspace/ptrans/
│   ├── app/backend/          ← Rust API (axum + Postgres + Redis + indexer)
│   ├── app/prover/           ← Rust + Sunspot CLI wrapper
│   ├── app/merkle/           ← Node.js + noir_js (Poseidon2)
│   ├── app/circuits/         ← Noir circuits (withdrawal, hash2, hashes)
│   └── app/frontend/         ← Vue 3 + Vite
├── docs/                     ← architecture, threat-model, zk-explained, deployment
│   └── ru/                   ← Russian docs
├── grafana/                  ← dashboards + datasources provisioning
├── docker-compose.yml
├── Dockerfile                ← solana-dev image
├── prometheus.yml
├── start-all.sh
└── PROJECT_CONTEXT.md        ← context for AI assistant
```

Full structure — in [`PROJECT_CONTEXT.md`](PROJECT_CONTEXT.md#4-структура-репозитория).

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Smart contract | Anchor (Rust) |
| ZK circuit | Noir |
| ZK proof | Sunspot → Groth16 (BN254) |
| On-chain verifier | gnark-solana |
| Backend | Rust + axum |
| DB | PostgreSQL 16 |
| Cache | Redis 7 |
| Frontend | Vue 3 + Vite + Pinia |
| Contracts client | Codama (autogen) |
| Monitoring | Prometheus + Grafana |
| Container | Docker + Docker Compose |

---

## Commands

```bash
# All services
./start-all.sh                     # start (tmux + docker)
./start-all.sh status              # status
./start-all.sh stop                # stop
./start-all.sh attach backend      # attach to tmux session
./start-all.sh logs backend        # tail log

# Program
cd workspace/ptrans
anchor build                       # build
anchor deploy --provider.cluster devnet  # deploy
anchor test                        # run tests

# Backend
cd workspace/ptrans/app/backend
cargo build                        # debug
cargo build --release              # release
cargo test                         # tests
cargo clippy                       # linter
cargo fmt                          # formatting
cargo doc --open                   # docs

# Frontend
cd workspace/ptrans/app/frontend
npm run dev                        # dev server
npm run build                      # production build
npm run test:unit                  # unit tests (vitest)
npm run test:e2e                   # e2e (Playwright)
npm run lint                       # eslint + oxlint
npm run codama                     # regenerate clients from IDL

# Merkle
cd workspace/ptrans/app/merkle
npm start                          # start service
npm test                           # tests
```

---

## Monitoring

After `./start-all.sh`:

- **Prometheus** (http://localhost:9090) — scrapes metrics from backend `/metrics` every 15 sec.
- **Grafana** (http://localhost:3000) — dashboard `ptrans Overview` with 9 panels:
  - HTTP Requests per Second (by endpoint and status)
  - HTTP Latency (p50 / p95)
  - Total Deposits / Withdrawals
  - Merkle Tree Size
  - Indexer Lag (seconds behind blockchain)
  - Merkle add_leaf Duration
  - Errors per Second
  - Internal Operations Duration

**Metrics:**

| Metric | What it shows |
|--------|---------------|
| `axum_http_requests_total` | HTTP requests |
| `axum_http_requests_duration_seconds` | HTTP latency |
| `ptrans_indexer_deposits_total` | Processed DepositEvent |
| `ptrans_indexer_withdrawals_total` | Processed WithdrawEvent |
| `ptrans_indexer_lag_seconds` | Lag behind blockchain |
| `ptrans_indexer_tick_duration_seconds` | One indexer tick duration |
| `ptrans_indexer_tree_size` | Tree size |
| `ptrans_tree_add_leaf_duration_seconds` | Add leaf duration |
| `ptrans_tree_hash_duration_seconds` | Poseidon2 hash duration |
| `ptrans_tree_errors_total` | Tree errors |
| `ptrans_db_errors_total` | DB errors |

---

## Security

- **Threat model:** [`docs/threat-model.md`](docs/threat-model.md)
- **Report vulnerabilities:** [`SECURITY.md`](SECURITY.md)

**Key invariants:**

1. Each `nullifier_hash` is used **exactly once** — guaranteed by
   `init` for the `NullifierRecord` PDA (`AccountAlreadyInUse` on repeat).
2. `root` must be in the history of `PoolAcc.roots[10]` — protects against proofs for an outdated tree.
3. ZK proof is verified **on-chain** via the verifier program — the backend cannot forge it.
4. `recipient` from the tx is checked against `to` from the instruction — the proof is bound to the recipient.

**Known limitations:**

- Groth16 setup — without a trusted ceremony.
- NullifierRecord is not deleted (rent is not returned).
- Backend sees commitments/nullifiers (but not the link between them).

**Automated checks:**

- `cargo-deny` — licenses, vulnerabilities, duplicates.
- `cargo-audit` — RUSTSEC vulnerability database.
- `npm audit` — Node.js dependency vulnerabilities.
- `trivy` — Docker image scanning.
- `gitleaks` — secret scanning in git history.
- GitHub Code Scanning — SARIF upload from Trivy.

---

## Documentation

- [`PROJECT_CONTEXT.md`](PROJECT_CONTEXT.md) — full project context for an AI assistant.
- [`docs/architecture.md`](docs/architecture.md) — detailed architecture.
- [`docs/zk-explained.md`](docs/zk-explained.md) — how the ZK part works.
- [`docs/threat-model.md`](docs/threat-model.md) — threat model.
- [`docs/deployment.md`](docs/deployment.md) — how to deploy.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — how to contribute.

**Russian version:**

- [`docs/ru/README.md`](docs/ru/README.md) — Russian version of README.

---

## License

MIT — see [`LICENSE`](LICENSE).
