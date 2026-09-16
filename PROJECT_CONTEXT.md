
# PROJECT_CONTEXT.md

> **Назначение:** этот файл — единая точка входа для AI-ассистента (Claude, ChatGPT и др.)
> в новый чат. Загрузи его первым — ассистент поймёт проект без копирования кода файлов.
>
> **Последнее обновление:** 2026-09-15

---

## 1. Что это за проект

**zk-pool** — приватный пул для переводов SOL на Solana с использованием ZK-доказательств
(Groth16 на BN254). Пользователь вносит SOL в общий vault, получает deposit note
(секреты) и может позже вывести SOL на любой адрес, не раскрывая связь между
депозитом и выводом.

Проект — portfolio-grade демонстрация полного цикла разработки ZK-протокола на Solana.

**Репозиторий:** <https://github.com/kwebhub/private-transfer>
**Лицензия:** MIT
**Сеть:** Solana Devnet

---

## 2. Архитектура

┌──────────────────────────────────────────────────────────────────┐
│ FRONTEND (Vue 3 + Vite + Pinia) │
│ • Deposit: генерирует commitment через noir_js, отправляет tx │
│ • Withdraw: получает commitments и proof из backend, генерирует │
│ witness, отправляет tx │
└──────────────────────────────────────────────────────────────────┘
↓ HTTP ↑ HTTP
┌──────────────────────────────────────────────────────────────────┐
│ BACKEND (Rust + axum) │
│ • /api/commitments — из Postgres, кеш в Redis │
│ • /api/root — из Redis tree │
│ • /api/proof — из Redis tree │
│ • /api/withdraw — проксирует в prover │
│ • Indexer — слушает события, пишет в БД, обновляет дерево │
│ • Rate limiting (Redis INCR + EXPIRE) │
│ • Prometheus metrics на /metrics │
└──────────────────────────────────────────────────────────────────┘
↓ ↓
┌──────────────────┐ ┌──────────────────────┐ ┌──────────────────────┐
│ POSTGRES │ │ REDIS │ │ MERKLE (Node.js) │
│ • commitments │ │ • кеш 30 сек │ │ • Noir Poseidon2 │
│ • roots │ │ • Merkle tree кеш │ │ • /hash, /root, │
│ • nullifiers │ │ • rate limit │ │ /proof │
└──────────────────┘ └──────────────────────┘ └──────────────────────┘
↑ HTTP
┌──────────────────────────────────────────────────────────────┴───┐
│ SOLANA (Anchor program ptrans) │
│ • pool: PDA ["pool3"] — PoolAcc │
│ • vault: PDA ["vault3", pool] — SystemAccount │
│ • nullifier_record: PDA ["nullifier_record", pool, nullifier] │
│ • verifier: on-chain Groth16 verifier (Sunspot) │
└──────────────────────────────────────────────────────────────────┘

### Сервисы и порты

| Сервис | Порт | Где запускается | Назначение |
| -------- | ------ | ----------------- | ------------ |
| frontend | 5173 | хост | Vue 3 + Vite |
| backend | 4001 | хост | Rust + axum API |
| prover | 4002 | контейнер `solana-dev` | Sunspot CLI wrapper |
| merkle | 4003 | хост | Node.js + noir_js |
| postgres | 5432 | Docker | БД |
| redis | 6379 | Docker | кеш |
| prometheus | 9090 | Docker | метрики |
| grafana | 3000 | Docker | визуализация |
| solana-dev | — | Docker | окружение Solana/Rust/Sunspot |

---

## 3. Технологии

| Слой | Технология | Версия | Зачем |
| ------ | ----------- | -------- | ------- |
| Smart contract | Anchor | 0.30+ | Solana program framework |
| ZK circuit | Noir | 1.0.0-rc.1 | DSL для ZK-схем |
| ZK backend | Sunspot | 1.0.0 | Noir → Groth16 для Solana (Go-бинарь, ELF с `.gopclntab`) |
| ZK verification | gnark-solana | — | On-chain verifier |
| Backend | Rust | 1.89+ | Производительность, типобезопасность |
| Backend framework | axum | 0.7 | HTTP API |
| Frontend | Vue 3 | 3.5+ | Composition API |
| Frontend build | Vite | 8.x | Быстрая сборка |
| State | Pinia | 4.x | Store |
| Contracts client | Codama | latest | Автогенерация из IDL |
| DB | PostgreSQL | 16 | Источник истины |
| Cache | Redis | 7 | Кеш, rate limit, Merkle tree |
| Monitoring | Prometheus + Grafana | latest | Метрики |
| Tests | cargo test, vitest, Playwright | — | Unit/E2E |

---

## 4. Структура репозитория

private-transfer/
├── PROJECT_CONTEXT.md ← этот файл
├── README.md
├── LICENSE ← MIT
├── CONTRIBUTING.md
├── SECURITY.md
├── CHANGELOG.md
├── .env.example
├── .gitignore
├── .editorconfig
├── .dockerignore
├── docker-compose.yml
├── Dockerfile ← solana-dev
├── prometheus.yml
├── start-all.sh ← запуск всех сервисов в tmux
├── Anchor.toml
├── Cargo.toml ← workspace (programs/*)
├── grafana/
│ └── provisioning/
│ ├── dashboards/
│ │ ├── dashboards.yml
│ │ └── ptrans.json
│ └── datasources/
│ └── prometheus.yml
├── programs/
│ └── ptrans/ ← Solana program
│ ├── Cargo.toml
│ ├── src/
│ │ ├── lib.rs
│ │ ├── constants.rs
│ │ ├── error.rs
│ │ ├── events.rs
│ │ ├── instructions.rs
│ │ ├── state.rs
│ │ └── instructions/
│ │ ├── deposit.rs
│ │ ├── pool.rs
│ │ └── withdraw.rs
│ └── tests/
├── workspace/ptrans/app/
│ ├── backend/ ← Rust API
│ │ ├── Cargo.toml
│ │ ├── migrations/
│ │ │ └── 001_init.sql
│ │ └── src/
│ │ ├── main.rs
│ │ ├── cache.rs
│ │ ├── config.rs
│ │ ├── db.rs
│ │ ├── indexer.rs
│ │ ├── metrics.rs
│ │ ├── rate_limit.rs
│ │ └── tree.rs
│ ├── prover/ ← Rust + Sunspot CLI
│ │ ├── Cargo.toml
│ │ └── src/main.rs
│ ├── merkle/ ← Node.js
│ │ ├── package.json
│ │ ├── src/server.js
│ │ └── circuits/
│ │ ├── hash2.json
│ │ └── withdrawal.json
│ ├── circuits/
│ │ ├── withdrawal/ ← Noir circuit
│ │ │ ├── Nargo.toml
│ │ │ ├── Prover.toml
│ │ │ └── src/
│ │ │ ├── main.nr
│ │ │ └── merkle_tree.nr
│ │ ├── hash2/ ← Poseidon2 hash(2)
│ │ └── hashes/ ← Poseidon2 hash(1), hash(3)
│ └── frontend/ ← Vue 3
│ ├── package.json
│ ├── vite.config.ts
│ ├── public/circuits/
│ │ ├── hash2.json
│ │ ├── hashes.json
│ │ └── withdrawal.json
│ └── src/
│ ├── components/
│ ├── composables/
│ │ ├── useDeposit.ts
│ │ ├── usePool.ts
│ │ ├── useWallet.ts
│ │ └── useWithdraw.ts
│ ├── generated/ ← Codama
│ ├── services/
│ │ ├── api.ts
│ │ ├── crypto.ts
│ │ └── poseidon.ts
│ ├── stores/
│ │ ├── deposits.ts
│ │ └── wallet.ts
│ └── views/
└── docs/
├── architecture.md
├── threat-model.md
├── zk-explained.md
└── deployment.md

---

## 5. Ключевые решения и почему

### 5.1. Почему PDA-на-нуллифаер вместо `Vec<[u8; 32]>`

**Было:** один аккаунт `NullifierSetAcc` с `Vec<[u8; 32]>` — лимит 256 записей.

**Стало:** отдельный `NullifierRecord` PDA для каждого нуллификатора:

- seeds: `["nullifier_record", pool, nullifier_hash]`
- `init` → если PDA существует, Anchor падает с `AccountAlreadyInUse` (double-spend protection)

**Плюсы:** неограниченное количество выводов, O(1) проверка, можно удалять старые.

### 5.2. Почему Groth16 + Sunspot, а не UltraHonk

Sunspot — **единственный** инструмент для Noir → Solana verifier на момент разработки.
Работает через Gnark (Go), генерирует Groth16 proof (324 байта для BN254).

### 5.3. Почему Poseidon2 считается через noir_js, а не через Rust

**Проблема:** `light-poseidon` и `poseidon-rs` дают **другие** хеши, чем Noir builtin
`std::hash::poseidon2_permutation`.

**Решение:** merkle-сервис на Node.js + `@noir-lang/noir_js`. Он использует **тот же**
`hash2.json`, что и фронт, и даёт **идентичные** хеши.

**Проверено:** `h(0,0)` = `18dfb8dc9b82229cff974efefc8df78b1ce96d9d844236b496785c698bc6732e`
совпадает с Noir.

### 5.4. Почему инкрементальное дерево в Redis

**Было:** на каждый `/api/proof` — построение всего дерева (2^20 = 1M hash-операций).

**Стало:** дерево кешируется в Redis (уровни + empty hashes). При новом листе —
обновляется **только путь** от листа до корня (20 hash-операций).

**Ключи в Redis:**

- `tree:{pool}:level:{d}` — JSON-массив хешей уровня `d`
- `tree:{pool}:empty:{d}` — "пустой" хеш для уровня `d`
- `tree:{pool}:root` — текущий корень
- `tree:{pool}:size` — количество листьев

### 5.5. Почему `reduceToField` для recipient

Pubkey Solana (32 байта) может **превышать** модуль поля BN254 (2^254).
Noir требует, чтобы Field-параметры были **меньше** модуля.
Поэтому `recipient` приводится к модулю **и на фронте** (для witness),
**и в программе** (в `encode_public_inputs`).

### 5.6. Почему rate limiting через Redis

Простой `INCR` + `EXPIRE` — атомарно, не требует внешних крейтов.
Лимиты: 5 req/min на `/api/withdraw`, 60 req/min на чтение.

---

## 6. Поток данных

### 6.1. Инициализация пула

1. Пользователь нажимает "Initialize Pool" в UI.
2. Frontend вызывает `getPoolInstructionAsync`.
3. Программа создаёт 3 PDA: `pool`, `vault`, (nullifier_record создаётся при первом выводе).
4. Транзакция подписана через `signTransaction` + `sendRawTransaction`.

### 6.2. Депозит

1. Frontend генерирует `nullifierSecret`, `secret` (32 случайных байта каждый).
2. Приводит их к модулю BN254 (reduction mod p).
3. Вычисляет `commitment = Poseidon2(nullifierSecret, secret, amount)` через `hashes.json`.
4. Запрашивает существующие commitments через `/api/commitments`.
5. Запрашивает новый root через `merkle:/root` (передаёт все commitments + новый).
6. Отправляет `deposit` tx: `[commitment, new_root, amount]`.
7. Сохраняет note в localStorage.
8. Indexer на backend ловит `DepositEvent` → пишет в Postgres → обновляет дерево в Redis.

### 6.3. Вывод

1. Frontend парсит note → `nullifierSecret`, `secret`.
2. Запрашивает commitments через `/api/commitments`.
3. Находит `leaf_index` для своего commitment.
4. Запрашивает Merkle proof через `/api/proof?leaf_index=N`.
5. Вычисляет `nullifierHash = Poseidon2(nullifierSecret)`.
6. Генерирует witness через `withdrawal.json` (noir_js).
7. Отправляет witness в `/api/withdraw` → backend проксирует в prover → Sunspot CLI.
8. Получает Groth16 proof (324 байта).
9. Собирает `withdraw` tx: `[proof, nullifier_hash, root, to, amount]`.
10. Программа: проверяет root в истории, создаёт `NullifierRecord` PDA,
    вызывает verifier (CPI), переводит SOL из vault.

---

## 7. ZK-схема

### Файл `circuits/withdrawal/src/main.nr`

**Публичные входы:**

- `root` — корень Merkle Tree
- `nullifier_hash` — Poseidon2(nullifier)
- `recipient` — Pubkey получателя (приведён к Field)
- `amount` — сумма в lamports

**Приватные входы:**

- `nullifier` — секрет из note
- `secret` — секрет из note
- `merkle_proof[20]` — siblings
- `is_even[20]` — флаги сторон

**Проверки:**

1. `commitment = Poseidon2(nullifier, secret, amount)`
2. `nullifier_hash == Poseidon2(nullifier)`
3. `computed_root == root` (через Merkle proof)

### Файл `circuits/hash2/src/main.nr`

Утилитарный circuit: `hash_2(left, right) = Poseidon2(left, right, 0, 0)[0]`.
Используется в merkle-сервисе для вычисления хешей.

### Файл `circuits/hashes/src/main.nr`

Утилитарный circuit: возвращает `(commitment, nullifier_hash)` одним вызовом.
Используется при депозите.

---

## 8. Solana program

### Программа `ptrans`

**Program ID:** `FbXJSZ171dcnHJVrd5E6KwvXAx7bMgxC44McF84vJ6cK`

**Verifier Program ID:** `EewognjaJhZUQgP59BrCx5SaJcsFQ6sn65FEwZ2FdZhg`

### Инструкции

| Инструкция | Что делает |
| ----------- | ----------- |
| `pool` | Инициализирует `PoolAcc` и `vault` |
| `deposit(commitment, new_root, amount)` | Переводит SOL в vault, эмитит `DepositEvent` |
| `withdraw(proof, nullifier_hash, root, to, amount)` | Проверяет proof, создаёт `NullifierRecord`, переводит SOL из vault |

### Аккаунты

- **PoolAcc** (PDA `["pool3"]`): `authority`, `next_leaf_index`, `total_deposits`, `current_root_index`, `roots[10]`.
- **Vault** (PDA `["vault3", pool]`): SystemAccount, хранит SOL.
- **NullifierRecord** (PDA `["nullifier_record", pool, nullifier_hash]`): `pool`, `nullifier_hash`, `recipient`, `amount`, `timestamp`.

### Events

- **DepositEvent** (88 байт): `discriminator(8) + commitment(32) + leaf_index(8) + timestamp(8) + new_root(32)`.
- **WithdrawEvent** (80 байт): `discriminator(8) + nullifier_hash(32) + recipient(32) + timestamp(8)`.

---

## 9. Backend API

| Метод | Путь | Лимит | Что делает |
| ------- | ------ | ------- | ----------- |
| GET | `/api/health` | без | Health check (+ DB status) |
| GET | `/api/commitments?pool_address=X` | 60/min | Список commitments из БД (кеш 30 сек) |
| GET | `/api/root?pool_address=X` | 60/min | Текущий root из Redis tree |
| GET | `/api/proof?pool_address=X&leaf_index=N` | 60/min | Merkle proof из Redis tree |
| POST | `/api/withdraw` | 5/min | Проксирует witness в prover |
| GET | `/metrics` | без | Prometheus metrics |

---

## 10. Frontend composables

- **`useWallet.ts`** — connect/disconnect, getProvider (Phantom/Solflare), refreshBalance.
- **`useDeposit.ts`** — генерация секретов, commitment, отправка deposit tx.
- **`useWithdraw.ts`** — запрос proof из backend, witness generation, отправка withdraw tx.
- **`usePool.ts`** — fetch pool info, initialize pool.

**Ключевой сервис — `services/poseidon.ts`:** обёртка над `@noir-lang/noir_js` с кешированием
`Noir` инстансов для `hash2.json`, `hashes.json`, `withdrawal.json`.

---

## 11. Команды

### Запуск

```bash
./start-all.sh              # запустить все сервисы (tmux + docker)
./start-all.sh status       # статус
./start-all.sh stop         # остановить всё
./start-all.sh attach backend  # подключиться к tmux
```

### Сборка

```bash
# Программа
cd ~/Projects/private-transfer/workspace/ptrans
anchor build && anchor deploy --provider.cluster devnet

# Frontend
cd app/frontend
npm run build

# Backend
cd app/backend
cargo build --release

# Merkle
cd app/merkle
npm start
```

### Codama (после изменения IDL)

```bash
cd app/frontend
npm run codama
```

---

## 12. Переменные окружения

workspace/ptrans/.env

```text
SOLANA_RPC_URL=https://api.devnet.solana.com
SOLANA_WALLET=~/.config/solana/id.json
PROGRAM_ID=FbXJSZ171dcnHJVrd5E6KwvXAx7bMgxC44McF84vJ6cK
VERIFIER_PROGRAM_ID=EewognjaJhZUQgP59BrCx5SaJcsFQ6sn65FEwZ2FdZhg
PORT=4001
RUST_LOG=info
DATABASE_URL=postgres://ptrans:ptrans_dev_password@localhost:5432/ptrans
REDIS_URL=redis://localhost:6379
POOL_ADDRESS=TQYnBSBF3z3FxMHupvt3cbKYXqsorYY9gYb19rjCncN
```

workspace/ptrans/app/frontend/.env

```text
VITE_SOLANA_RPC_URL=https://api.devnet.solana.com
VITE_PROGRAM_ID=FbXJSZ171dcnHJVrd5E6KwvXAx7bMgxC44McF84vJ6cK
VITE_VERIFIER_PROGRAM_ID=EewognjaJhZUQgP59BrCx5SaJcsFQ6sn65FEwZ2FdZhg
VITE_API_URL=http://localhost:4001
VITE_MERKLE_URL=http://localhost:4003
VITE_PAYER=4aPAEEmJdLz4fYE44Krad3MP8Q1wAUScg3zMuC8fcBur
VITE_MERKLE_TREE_DEPTH=20
VITE_MIN_DEPOSIT=0.01
```

---

## 13. Git workflow

- Conventional commits: feat:, fix:, docs:, chore:, refactor:, test:, ci:.
- Ветки: main (protected), feature-ветки feat/..., fix/....
- PR: обязательный, с описанием, тестами, ревью.
- Releases: semver, changelog.
- CI: GitHub Actions — build + test для программы, backend, frontend.

---

## 14. Roadmap

- [ ] Anchor program (pool/deposit/withdraw)
- [ ] Noir circuit + Sunspot Groth16
- [ ] Backend (axum + Postgres + Redis)
- [ ] Indexer + инкрементальное Merkle tree
- [ ] Rate limiting
- [ ] Prometheus + Grafana
- [ ] start-all.sh
- [ ] CI/CD (GitHub Actions)
- [ ] Документация (docs/)
- [ ] Тесты (unit + integration + e2e)
- [ ] Рефакторинг (tracing, thiserror, doc-comments)
- [ ] README с скриншотами и бейджами
- [ ] GitHub flow (issue/PR templates, branch protection)
- [ ] Threat model + security checklist

### Масштабирование (post-MVP)

- [ ] **Очередь задач на Redis Streams для prover.**
  Проблема: `sunspot prove` — однопоточный CLI, 30-60 сек на proof.
  Решение: Redis Streams (`proof_queue` + `proof_results:{request_id}`).
  Плюсы: параллелизм, at-least-once delivery, ноль новой инфраструктуры.
  Почему не Kafka: избыточна для десятков запросов в час.
- [ ] **Горизонтальное масштабирование prover.**
  2-3 воркера, читающих из `proof_queue` через `XREADGROUP`.
- [ ] **Мониторинг очереди.**
  Метрики: `ptrans_prover_queue_depth`, `ptrans_prover_processing_time`.

---

## 15. Безопасность (TODO)

- Threat model (docs/threat-model.md)
- Аудит-чеклист
- Инварианты программы
- Тесты на double-spend, invalid proof, invalid root

---

## 16. Известные ограничения

- Trusted setup для Groth16. Sunspot использует стандартный setup без ceremony.
- Для production нужен trusted-setup от Reilabs.
- NullifierRecord не удаляется. После withdraw аккаунт остаётся, rent (~0.003 SOL) не возвращается.
- MerkleTree depth 20 — максимум 1M листьев.
- Zero-knowledge only for on-chain data. Backend видит commitments/nullifiers.
