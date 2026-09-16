<!-- Markdownlint-disable MD013 -->
# zk-pool

> Private SOL transfers on Solana using Groth16 zero-knowledge proofs.

[![CI](https://github.com/kwebhub/private-transfer/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kwebhub/private-transfer/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana)](https://solana.com)
[![Noir](https://img.shields.io/badge/Noir-1.0.0--rc.1-blue)](https://noir-lang.org)

**zk-pool** — приватный пул для переводов SOL. Пользователь вносит SOL в общий vault,
получает *deposit note* (два секрета), и позже может вывести SOL на **любой** адрес,
не раскрывая связь между депозитом и выводом.

Проект реализует полный стек ZK-протокола:

- **Noir** — ZK-circuit с Poseidon2 и Merkle proof.
- **Sunspot** — Noir → Groth16 proof для Solana.
- **Anchor** — Solana program с on-chain верификатором.
- **Rust + axum** — backend с индексером и Merkle tree в Redis.
- **Vue 3 + Vite** — frontend с witness generation на клиенте.
- **Postgres + Redis** — хранение и кеш.
- **Prometheus + Grafana** — мониторинг.

> ⚠️ **Статус:** devnet, portfolio project. Не использовать в mainnet без аудита.

---

## Содержание

- [Как это работает](#как-это-работает)
- [Архитектура](#архитектура)
- [Быстрый старт](#быстрый-старт)
- [Структура репозитория](#структура-репозитория)
- [Технологии](#технологии)
- [Команды](#команды)
- [Мониторинг](#мониторинг)
- [Безопасность](#безопасность)
- [Документация](#документация)
- [Лицензия](#лицензия)

---

## Как это работает

### Депозит

1. Frontend генерирует два секрета: `nullifierSecret`, `secret`.
2. Вычисляет `commitment = Poseidon2(nullifierSecret, secret, amount)`.
3. Запрашивает актуальный Merkle root у backend.
4. Отправляет `deposit` tx в программу: SOL уходит в vault, commitment становится листом дерева.
5. Сохраняет *deposit note* (секреты + commitment) в localStorage.

### Вывод

1. Frontend парсит note, запрашивает Merkle proof у backend.
2. Генерирует ZK-witness (noir_js) — доказательство, что он знает секреты,
   соответствующие листу в дереве, **не раскрывая** их.
3. Backend проксирует witness в prover (Sunspot), получает Groth16 proof.
4. Frontend отправляет `withdraw` tx: программа верифицирует proof on-chain
   через verifier program и переводит SOL из vault.
5. `NullifierRecord` PDA создаётся — повторный вывод с тем же nullifier невозможен.

**Что видит блокчейн:** commitment, nullifier_hash, root, amount.
**Что скрыто:** связь между депозитом и выводом.

Подробнее — [`docs/zk-explained.md`](docs/zk-explained.md).

---

## Архитектура

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
│  Indexer: слушает события → Postgres → Merkle tree          │
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

Подробнее — [`docs/architecture.md`](docs/architecture.md).

---

## Быстрый старт

### Требования

- Linux (Debian 12+ / Ubuntu 22.04+)
- Docker + Docker Compose
- Rust 1.89+ ([rustup](https://rustup.rs))
- Node.js 22+ ([nvm](https://github.com/nvm-sh/nvm))
- pnpm (`npm install -g pnpm`)
- tmux (`sudo apt install tmux`)
- Anchor (`cargo install --git https://github.com/coral-xyz/anchor avm && avm install 0.30.1 && avm use 0.30.1`)
- Solana CLI (`sh -c "$(curl -sSfL https://release.solana.com/stable/install)"`)
- Noir (`curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash && noirup`)

### Установка

```bash
git clone https://github.com/kwebhub/private-transfer.git
cd private-transfer

# 1. Запустить Docker-окружение (solana-dev, postgres, redis, prometheus, grafana)
docker compose up -d

# 2. Собрать и задеплоить программу на devnet
cd workspace/ptrans
anchor build
anchor deploy --provider.cluster devnet
cd ../..

# 3. Скопировать .env.example → .env (в нужные места)
cp .env.example workspace/ptrans/.env
cp workspace/ptrans/.env workspace/ptrans/app/frontend/.env
# отредактировать значения под свой кошелёк

# 4. Применить миграции БД
docker compose exec -T postgres psql -U ptrans -d ptrans < workspace/ptrans/app/backend/migrations/001_init.sql

# 5. Установить зависимости frontend и merkle
cd workspace/ptrans/app/frontend && pnpm install && cd -
cd workspace/ptrans/app/merkle && pnpm install && cd -

# 6. Запустить все сервисы одной командой
./start-all.sh
```

После запуска:

- Frontend: <http://localhost:5173>
- Backend API: <http://localhost:4001>
- Merkle: <http://localhost:4003>
- Prometheus: <http://localhost:9090>
- Grafana: <http://localhost:3000> (admin / admin)

### Остановка

```bash
./start-all.sh stop
docker compose down
```

---

## Структура репозитория

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
├── grafana/                  ← dashboards + datasources provisioning
├── docker-compose.yml
├── Dockerfile                ← solana-dev image
├── prometheus.yml
├── start-all.sh
└── PROJECT_CONTEXT.md        ← контекст для AI-ассистента
```

Полная структура — в [`PROJECT_CONTEXT.md`](PROJECT_CONTEXT.md#4-структура-репозитория).

---

## Технологии

| Слой | Технология |
| ------ | ----------- |
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

## Команды

```bash
# Все сервисы
./start-all.sh                     # запустить (tmux + docker)
./start-all.sh status              # статус
./start-all.sh stop                # остановить
./start-all.sh attach backend      # подключиться к tmux-сессии
./start-all.sh logs backend        # tail лога

# Программа
cd workspace/ptrans
anchor build                       # собрать
anchor deploy --provider.cluster devnet  # задеплоить
anchor test                        # запустить тесты

# Backend
cd workspace/ptrans/app/backend
cargo build                        # debug
cargo build --release              # release
cargo test                         # тесты
cargo clippy                       # линтер
cargo fmt                          # форматирование

# Frontend
cd workspace/ptrans/app/frontend
npm run dev                        # dev-сервер
npm run build                      # production build
npm run test:unit                  # unit-тесты (vitest)
npm run test:e2e                   # e2e (Playwright)
npm run lint                       # eslint + oxlint
npm run codama                     # регенерация клиентов из IDL

# Merkle
cd workspace/ptrans/app/merkle
npm start                          # запустить сервис
npm test                           # тесты
```

---

## Мониторинг

После `./start-all.sh`:

- **Prometheus** (<http://localhost:9090>) — собирает метрики с `/metrics` backend'а каждые 15 сек.
- **Grafana** (<http://localhost:3000>) — дашборд `ptrans Overview` с 9 панелями:
  - HTTP Requests per Second (по endpoint и статусу)
  - HTTP Latency (p50 / p95)
  - Total Deposits / Withdrawals
  - Merkle Tree Size
  - Indexer Lag (сек от блокчейна)
  - Merkle add_leaf Duration
  - Errors per Second
  - Internal Operations Duration

**Метрики:**

| Метрика | Что показывает |
| --------- | --------------- |
| `axum_http_requests_total` | HTTP-запросы |
| `axum_http_requests_duration_seconds` | Латентность HTTP |
| `ptrans_indexer_deposits_total` | Обработано DepositEvent |
| `ptrans_indexer_withdrawals_total` | Обработано WithdrawEvent |
| `ptrans_indexer_lag_seconds` | Лаг от блокчейна |
| `ptrans_indexer_tick_duration_seconds` | Время одного тика индексера |
| `ptrans_indexer_tree_size` | Размер дерева |
| `ptrans_tree_add_leaf_duration_seconds` | Время добавления листа |
| `ptrans_tree_hash_duration_seconds` | Время Poseidon2 hash |
| `ptrans_tree_errors_total` | Ошибки дерева |
| `ptrans_db_errors_total` | Ошибки БД |

---

## Безопасность

- **Threat model:** [`docs/threat-model.md`](docs/threat-model.md) (TODO)
- **Report vulnerabilities:** [`SECURITY.md`](SECURITY.md)

**Ключевые инварианты:**

1. Каждый `nullifier_hash` используется **ровно один раз** — гарантируется
   `init` для `NullifierRecord` PDA (`AccountAlreadyInUse` при повторе).
2. `root` должен быть в истории `PoolAcc.roots[10]` — защита от proof для устаревшего дерева.
3. ZK-proof верифицируется **on-chain** через verifier program — backend не может подделать.
4. `recipient` из tx сверяется с `to` из instruction — proof привязан к получателю.

**Известные ограничения:**

- Groth16 setup — без trusted ceremony.
- NullifierRecord не удаляется (rent не возвращается).
- Backend видит commitments/nullifiers (но не связь между ними).

---

## Документация

- [`PROJECT_CONTEXT.md`](PROJECT_CONTEXT.md) — полный контекст проекта для AI-ассистента.
- [`docs/architecture.md`](docs/architecture.md) — детальная архитектура.
- [`docs/zk-explained.md`](docs/zk-explained.md) — как работает ZK-часть.
- [`docs/threat-model.md`](docs/threat-model.md) — модель угроз.
- [`docs/deployment.md`](docs/deployment.md) — как деплоить.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — как контрибьютить.

---

## Лицензия

MIT — см. [`LICENSE`](LICENSE).
