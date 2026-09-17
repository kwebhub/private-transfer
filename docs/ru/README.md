# zk-pool

> Приватные переводы SOL на Solana с использованием Groth16 zero-knowledge доказательств.

[![CI](https://github.com/kwebhub/private-transfer/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kwebhub/private-transfer/actions/workflows/ci.yml)
[![Security](https://github.com/kwebhub/private-transfer/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/kwebhub/private-transfer/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Solana](https://img.shields.io/badge/Solana-Devnet-9945FF?logo=solana)](https://solana.com)
[![Noir](https://img.shields.io/badge/Noir-1.0.0--rc.1-blue)](https://noir-lang.org)

**zk-pool** — приватный пул для переводов SOL. Пользователь вносит SOL в общий vault, получает *deposit note* (два секрета), и позже может вывести SOL на **любой** адрес, не раскрывая связь между депозитом и выводом.

[English version](../../README.md)

---

## Содержание

- [Как это работает](#как-это-работает)
- [Архитектура](#архитектура)
- [Быстрый старт](#быстрый-старт)
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

Подробнее — [`zk-explained.md`](zk-explained.md).

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

Подробнее — [`architecture.md`](architecture.md).

---

## Быстрый старт

### Требования

- Linux (Debian 12+ / Ubuntu 22.04+)
- Docker + Docker Compose
- Rust 1.89+
- Node.js 22+
- pnpm
- tmux
- Anchor 0.32.2
- Solana CLI
- Noir (nargo)

### Установка

```bash
git clone https://github.com/kwebhub/private-transfer.git
cd private-transfer

# 1. Запустить Docker-окружение
docker compose up -d

# 2. Собрать и задеплоить программу на devnet
cd workspace/ptrans
anchor build
anchor deploy --provider.cluster devnet
cd ../..

# 3. Скопировать .env
cp .env.example workspace/ptrans/.env
cp workspace/ptrans/.env.example workspace/ptrans/app/frontend/.env

# 4. Применить миграции БД
docker compose exec -T postgres psql -U ptrans -d ptrans \
  < workspace/ptrans/app/backend/migrations/001_init.sql

# 5. Установить зависимости
cd workspace/ptrans/app/frontend && pnpm install && cd -
cd workspace/ptrans/app/merkle && pnpm install && cd -

# 6. Запустить все сервисы
./start-all.sh
```

**Открыть:**

- Frontend: http://localhost:5173
- Backend API: http://localhost:4001
- Merkle: http://localhost:4003
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000

### Остановка

```bash
./start-all.sh stop
docker compose down
```

---

## Технологии

| Слой | Технология |
|------|-----------|
| Smart contract | Anchor (Rust) |
| ZK circuit | Noir |
| ZK proof | Sunspot → Groth16 (BN254) |
| On-chain verifier | gnark-solana |
| Backend | Rust + axum |
| DB | PostgreSQL 16 |
| Cache | Redis 7 |
| Frontend | Vue 3 + Vite + Pinia |
| Contracts client | Codama |
| Monitoring | Prometheus + Grafana |
| Container | Docker + Docker Compose |

---

## Команды

```bash
# Все сервисы
./start-all.sh                     # запустить (tmux + docker)
./start-all.sh status              # статус
./start-all.sh stop                # остановить
./start-all.sh attach backend      # tmux-сессия backend
./start-all.sh logs backend        # tail лога

# Программа
cd workspace/ptrans
anchor build                       # собрать
anchor deploy --provider.cluster devnet  # задеплоить
cargo test                         # тесты программы

# Backend
cd workspace/ptrans/app/backend
cargo build                        # debug
cargo build --release              # release
cargo test                         # тесты
cargo clippy                       # линтер
cargo fmt                          # форматирование
cargo doc --open                   # документация

# Frontend
cd workspace/ptrans/app/frontend
npm run dev                        # dev-сервер
npm run build                      # production build
npx playwright test                # e2e-тесты
npm run codama                     # регенерация клиентов

# Merkle
cd workspace/ptrans/app/merkle
npm start                          # запустить
npm test                           # тесты
```

---

## Мониторинг

После `./start-all.sh`:

- **Prometheus** (http://localhost:9090) — собирает метрики с `/metrics` backend'а каждые 15 сек.
- **Grafana** (http://localhost:3000) — дашборд `ptrans Overview` с 9 панелями.

**Метрики:**

| Метрика | Что показывает |
|---------|---------------|
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

- **Threat model:** [`threat-model.md`](threat-model.md)
- **Report vulnerabilities:** [`SECURITY.md`](../../SECURITY.md)

**Ключевые инварианты:**

1. Каждый `nullifier_hash` используется **ровно один раз**.
2. `root` должен быть в истории `PoolAcc.roots[10]`.
3. ZK-proof верифицируется **on-chain**.
4. `recipient` привязан к proof.

**Автоматические проверки:**

- `cargo-deny` — лицензии, уязвимости, дубликаты.
- `cargo-audit` — RUSTSEC-база.
- `npm audit` — Node.js-зависимости.
- `trivy` — Docker-образы.
- `gitleaks` — секреты в git history.

---

## Документация

- [`architecture.md`](architecture.md) — детальная архитектура.
- [`zk-explained.md`](zk-explained.md) — как работает ZK-часть.
- [`threat-model.md`](threat-model.md) — модель угроз.
- [`deployment.md`](deployment.md) — как деплоить.

---

## Лицензия

MIT — см. [`LICENSE`](../../LICENSE).
