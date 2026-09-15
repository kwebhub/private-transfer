# Architecture

> Детальная архитектура zk-pool: компоненты, потоки данных, взаимодействие сервисов.

## Содержание

- [Обзор](#обзор)
- [Диаграмма](#диаграмма)
- [Компоненты](#компоненты)
- [Потоки данных](#потоки-данных)
- [Хранилища](#хранилища)
- [Сеть и порты](#сеть-и-порты)
- [Отказоустойчивость](#отказоустойчивость)

---

## Обзор

zk-pool — **микросервисная** система. Каждый сервис отвечает за свою часть:

| Сервис | Ответственность | Технология |
| -------- | ---------------- | ------------ |
| **frontend** | UI, генерация witness, отправка tx | Vue 3 + Vite + Pinia |
| **backend** | API, индексер, кеш, rate limiting | Rust + axum |
| **prover** | Генерация Groth16-proof | Rust + Sunspot CLI |
| **merkle** | Poseidon2 hash, Merkle tree | Node.js + noir_js |
| **postgres** | Источник истины (commitments, nullifiers, roots) | PostgreSQL 16 |
| **redis** | Кеш, Merkle tree, rate limit | Redis 7 |
| **prometheus** | Сбор метрик | Prometheus |
| **grafana** | Визуализация | Grafana |
| **solana-dev** | Окружение Rust/Solana/Sunspot | Ubuntu + Rust + Anchor + Nargo |

**Ключевые принципы:**

1. **Микросервисы** — каждый можно масштабировать отдельно.
2. **Изоляция** — `backend` не зависит от `solana-sdk`, `prover` не зависит от `sqlx`.
3. **Кеш** — тяжёлые операции кешируются в Redis.
4. **Stateless API** — backend не хранит состояние в памяти (кроме кеша).

---

## Диаграмма

```
┌─────────────────────────────────────────────────────────────────┐
│  Browser (Vue 3 + Phantom/Solflare)                            │
│                                                                 │
│  useDeposit: generateSecrets → computeCommitment → sendTx      │
│  useWithdraw: getProof → generateWitness → sendTx              │
└─────────────────────────────────────────────────────────────────┘
       ↓ HTTP                         ↑ HTTP
┌─────────────────────────────────────────────────────────────────┐
│  Backend (Rust + axum) — порт 4001                              │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ /api/        │  │ /api/        │  │ /api/                │  │
│  │ commitments  │  │ root         │  │ proof                │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────┘  │
│         │                 │                     │              │
│         ▼                 ▼                     ▼              │
│  ┌─────────────────────────────────────────────────┐          │
│  │  Cache layer (Redis)                            │          │
│  │  commitments:{pool}  · tree:{pool}:*            │          │
│  └─────────────────────────────────────────────────┘          │
│                                                                 │
│  ┌─────────────────────────────────────────────────┐          │
│  │  Indexer (фоновая задача)                        │          │
│  │  getSignaturesForAddress → parse events → DB    │          │
│  │  → update Merkle tree in Redis                  │          │
│  └─────────────────────────────────────────────────┘          │
│                                                                 │
│  ┌─────────────────────────────────────────────────┐          │
│  │  Rate limiter (Redis INCR + EXPIRE)             │          │
│  └─────────────────────────────────────────────────┘          │
│                                                                 │
│  ┌─────────────────────────────────────────────────┐          │
│  │  Prometheus metrics на /metrics                 │          │
│  └─────────────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────────────┘
       ↓              ↓              ↓              ↑
┌────────────┐ ┌────────────┐ ┌────────────┐ ┌─────────────┐
│ PostgreSQL │ │   Redis    │ │  Merkle    │ │  Prover     │
│  :5432     │ │   :6379    │ │  :4003     │ │  :4002      │
│            │ │            │ │            │ │             │
│ commitments│ │ tree:level │ │ Poseidon2  │ │ Sunspot CLI │
│ roots      │ │ empty      │ │ (noir_js)  │ │ Groth16     │
│ nullifiers │ │ root,size  │ │            │ │             │
│            │ │ rate:limit │ │            │ │             │
└────────────┘ └────────────┘ └────────────┘ └─────────────┘
                                                    ↑
                                                    │
┌───────────────────────────────────────────────────┴─────────────┐
│  Solana (devnet)                                                │
│                                                                 │
│  Program: FbXJSZ171dcnHJVrd5E6KwvXAx7bMgxC44McF84vJ6cK        │
│  Verifier: EewognjaJhZUQgP59BrCx5SaJcsFQ6sn65FEwZ2FdZhg       │
│                                                                 │
│  PoolAcc PDA (["pool3"])                                        │
│  Vault PDA (["vault3", pool])                                   │
│  NullifierRecord PDA (["nullifier_record", pool, nullifier])    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Компоненты

### Frontend

**Стек:** Vue 3 (Composition API) + Vite + Pinia + TypeScript.

**Ключевые composables:**

- **`useWallet`** — connect/disconnect, getProvider (Phantom/Solflare), refreshBalance.
- **`useDeposit`** — генерация секретов, commitment, отправка deposit tx.
- **`useWithdraw`** — запрос proof, witness generation, отправка withdraw tx.
- **`usePool`** — fetch pool info, initialize pool.

**Ключевые services:**

- **`poseidon.ts`** — обёртка над `@noir-lang/noir_js` с кешированием `Noir` инстансов.
- **`crypto.ts`** — генерация секретов (с reduction mod p), hex ↔ bytes.
- **`api.ts`** — HTTP-клиент к backend.
- **`storage.ts`** — работа с localStorage (deposit notes).

**Почему witness генерируется на клиенте:**

Приватность. Если witness отправлять на backend, он видит `nullifierSecret` и `secret`
→ backend может связать депозит с выводом. Генерация witness на клиенте —
**единственный** способ сохранить приватность.

### Backend

**Стек:** Rust + axum + sqlx + redis + reqwest + metrics.

**Модули:**

- **`main.rs`** — bootstrap, роуты, middleware, Prometheus recorder.
- **`db.rs`** — обёртка над Postgres (commitments, roots, nullifiers).
- **`cache.rs`** — обёртка над Redis (кеш, tree, rate limit).
- **`tree.rs`** — инкрементальное Merkle tree в Redis.
- **`indexer.rs`** — фоновая задача: слушает события через RPC, пишет в БД, обновляет дерево.
- **`rate_limit.rs`** — middleware rate limiting.
- **`metrics.rs`** — регистрация кастомных метрик.
- **`config.rs`** — конфигурация (TODO).

**API:**

| Метод | Путь | Лимит | Описание |
| ------- | ------ | ------- | ---------- |
| GET | `/api/health` | без | Health check + DB status |
| GET | `/api/commitments?pool_address=X` | 60/min | Список commitments |
| GET | `/api/root?pool_address=X` | 60/min | Текущий root |
| GET | `/api/proof?pool_address=X&leaf_index=N` | 60/min | Merkle proof |
| POST | `/api/withdraw` | 5/min | Проксирует witness → prover |
| GET | `/metrics` | без | Prometheus metrics |

**Индексер — ключевая часть:**

1. Каждые 5 секунд вызывает `getSignaturesForAddress(pool, limit=100, before=X)`.
2. Идёт по пагинации, пока не дойдёт до `last_signature` (в Redis).
3. Разворачивает — от старых к новым.
4. Для каждой tx парсит логи: `DepositEvent` (88 байт) и `WithdrawEvent` (80 байт).
5. **Дедупликация:** `commitment_exists(leaf_index)` и `is_nullifier_used(hash)`.
6. При новом депозите — `tree::add_leaf()` обновляет дерево в Redis.
7. Запоминает `last_signature` в Redis.

### Prover

**Стек:** Rust + tokio + axum. **Живёт внутри `solana-dev` контейнера.**

**Зачем отдельный сервис:**

- `sunspot prove` — долгая операция (30-60 сек).
- Нельзя блокировать backend.
- Отдельный HTTP API позволяет добавить очередь, кеш, retry.

**API:**

- `POST /prove` — принимает `{ witness: base64 }`, возвращает `{ proof, public_witness }`.
- `GET /health` — health check.

**Как работает:**

1. Декодирует witness из base64.
2. Сохраняет во временный файл `/tmp/witness-UUID.gz`.
3. Вызывает `sunspot prove withdrawal.json witness.gz withdrawal.ccs withdrawal.pk`.
4. Читает `withdrawal.proof` (324 байта) и `withdrawal.pw` (140 байт).
5. Кодирует в base64 и возвращает.
6. Удаляет временный файл.

### Merkle

**Стек:** Node.js + Fastify + `@noir-lang/noir_js`.

**Зачем отдельный сервис:**

`light-poseidon` и `poseidon-rs` дают **другие** хеши, чем Noir builtin.
Единственный способ получить **идентичные** хеши — использовать **тот же**
`hash2.json`, что и Noir. А `noir_js` работает **только** в JS.

**API:**

- `POST /hash` — `{ left, right }` → `{ hash }`.
- `POST /root` — `{ commitments: [hex] }` → `{ root }`.
- `POST /proof` — `{ commitments, leaf_index }` → `{ proof, is_even, root }`.
- `GET /health`.

### Postgres

**Источник истины.** Схема:

```sql
commitments (id, leaf_index, commitment, pool_address, tx_signature, created_at)
roots (id, root, leaf_index, pool_address, tx_signature, created_at)
nullifiers (nullifier_hash PK, pool_address, recipient, amount, tx_signature, used_at)
```

**Индексы:**

- `commitments(leaf_index)` — уникальный.
- `commitments(pool_address)` — для фильтрации.
- `roots(root)` — для поиска по корню.
- `nullifiers(pool_address)` — для агрегации.

### Redis

**Кеш + Merkle tree + rate limit.**

**Ключи:**

| Ключ | TTL | Назначение |
| ------ | ----- | ----------- |
| `commitments:{pool}` | 30s | Кеш списка commitments |
| `root:{pool}` | 30s | Кеш текущего root (legacy) |
| `tree:{pool}:level:{d}` | ∞ | JSON-массив хешей уровня `d` |
| `tree:{pool}:empty:{d}` | ∞ | "Пустой" хеш для уровня `d` |
| `tree:{pool}:root` | ∞ | Текущий корень |
| `tree:{pool}:size` | ∞ | Количество листьев |
| `indexer:{pool}:last_signature` | ∞ | Последняя обработанная tx |
| `ratelimit:{endpoint}:{ip}` | 60s | Счётчик запросов |

---

## Потоки данных

### Депозит (полный flow)

```
1. User → Frontend: вводит сумму
2. Frontend: generateSecrets() → { nullifierSecret, secret }
3. Frontend: reduceToField(nullifierSecret, secret)
4. Frontend: noir_js.execute(hashes.json) → { commitment, nullifierHash }
5. Frontend → Backend: GET /api/commitments?pool_address=X
6. Backend → Redis: GET commitments:{pool}
   - hit → return
   - miss → Backend → Postgres: SELECT ... FROM commitments
             Backend → Redis: SETEX commitments:{pool} 30s
7. Frontend → Merkle: POST /root { commitments: [...existing, new] }
8. Merkle → Frontend: { root }
9. Frontend → Solana: tx [deposit(commitment, new_root, amount)]
10. Solana: PoolAcc.next_leaf_index += 1
           emit DepositEvent
11. Frontend → localStorage: сохраняет note
12. Indexer (каждые 5s): getSignaturesForAddress → находит DepositEvent
13. Indexer → Postgres: INSERT commitments
14. Indexer → Merkle: POST /hash × 20 (обновление пути)
15. Indexer → Redis: SET tree:{pool}:level:{d}, root, size
16. Indexer → Redis: SET indexer:{pool}:last_signature
```

### Вывод (полный flow)

```
1. User → Frontend: вставляет note, адрес получателя, сумму
2. Frontend: parse note → { nullifierSecret, secret }
3. Frontend: computeNullifierHash(nullifierSecret)
4. Frontend: localStorage lookup по nullifierHash
5. Frontend → Backend: GET /api/commitments
6. Frontend: находит leaf_index для своего commitment
7. Frontend → Backend: GET /api/proof?leaf_index=N
8. Backend → Redis: GET tree:{pool}:level:{d}
9. Backend → Frontend: { proof[20], is_even[20], root }
10. Frontend: noir_js.execute(withdrawal.json) → witness
11. Frontend → Backend: POST /api/withdraw { witness: base64 }
12. Backend → Prover: POST /prove { witness: base64 }
13. Prover: sunspot prove → { proof, public_witness }
14. Prover → Backend: { proof, public_witness }
15. Backend → Frontend: { proof, public_witness }
16. Frontend → Solana: tx [withdraw(proof, nullifier_hash, root, to, amount)]
17. Solana: verifier (CPI) → "Proof verified successfully!"
18. Solana: create NullifierRecord PDA
           transfer vault → recipient
19. Indexer: находит WithdrawEvent
20. Indexer → Postgres: INSERT nullifiers
```

---

## Хранилища

| Хранилище | Что хранит | Почему |
| ----------- | ----------- | -------- |
| **Postgres** | commitments, roots, nullifiers | Источник истины, транзакции, надёжность |
| **Redis** | кеш, tree, rate limit | Скорость, TTL, атомарные операции |
| **localStorage** | deposit notes | Приватность — секреты остаются в браузере |
| **Solana** | PoolAcc, Vault, NullifierRecord | Публичный реестр |

---

## Сеть и порты

| Сервис | Порт | Протокол | Где |
| -------- | ------ | ---------- | ----- |
| frontend | 5173 | HTTP | хост |
| backend | 4001 | HTTP | хост |
| prover | 4002 | HTTP | контейнер solana-dev (network_mode: host) |
| merkle | 4003 | HTTP | хост |
| postgres | 5432 | TCP | Docker |
| redis | 6379 | TCP | Docker |
| prometheus | 9090 | HTTP | Docker |
| grafana | 3000 | HTTP | Docker |

**Связь backend ↔ prover:**

- `backend` использует `PROVER_URL=http://localhost:4002`.
- Так как `solana-dev` в `network_mode: host`, `localhost:4002` работает.

**Связь prometheus ↔ backend:**

- `prometheus` в Docker, `backend` на хосте.
- Используется `host.docker.internal:4001` + `extra_hosts: host-gateway`.

---

## Отказоустойчивость

**Что упадёт, если сервис недоступен:**

| Сервис упал | Последствия |
| ------------- | ------------ |
| frontend | Ничего не работает (но это UI) |
| backend | API недоступно, но blockchain работает |
| prover | Withdraw не работает (proof generation) |
| merkle | Депозит не работает (новый root), withdraw работает (proof из кеша) |
| postgres | Backend падает |
| redis | Backend теряет кеш, но работает через Postgres |
| Solana RPC | Indexer не синхронизируется, но blockchain работает |

**Что нужно добавить для production:**

1. **Replicas** — 2+ backend, 2+ prover.
2. **Очередь** — Redis Streams / RabbitMQ для запросов на proof.
3. **Circuit breaker** — для prover и merkle.
4. **Fallback RPC** — несколько эндпоинтов.
5. **Backup Postgres** — ежедневные снапшоты.
6. **Мониторинг алерты** — Grafana Alerting / Alertmanager.
