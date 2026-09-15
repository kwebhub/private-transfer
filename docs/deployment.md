# Deployment

> Пошаговая инструкция: как собрать и задеплоить zk-pool с нуля.

## Содержание

- [Требования](#требования)
- [Шаг 1: Клонирование](#шаг-1-клонирование)
- [Шаг 2: Docker-окружение](#шаг-2-docker-окружение)
- [Шаг 3: ZK-схема (Noir + Sunspot)](#шаг-3-zk-схема-noir--sunspot)
- [Шаг 4: Solana program (Anchor)](#шаг-4-solana-program-anchor)
- [Шаг 5: База данных](#шаг-5-база-данных)
- [Шаг 6: Backend](#шаг-6-backend)
- [Шаг 7: Merkle-сервис](#шаг-7-merkle-сервис)
- [Шаг 8: Prover](#шаг-8-prover)
- [Шаг 9: Frontend](#шаг-9-frontend)
- [Шаг 10: Мониторинг](#шаг-10-мониторинг)
- [Запуск всего](#запуск-всего)

---

## Требования

### Система

- Linux (Debian 12+ / Ubuntu 22.04+)
- 8 GB RAM минимум (16 GB рекомендуется)
- 20 GB свободного места

### Инструменты

```bash
# Docker
sudo apt install -y docker.io docker-compose-v2
sudo usermod -aG docker $USER
# перелогиниться

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Node.js через nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.7/install.sh | bash
source ~/.bashrc
nvm install node
npm install -g pnpm

# Anchor (через avm)
cargo install --git https://github.com/coral-xyz/anchor avm --force
avm install 0.30.1
avm use 0.30.1

# Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Noir
curl -L https://raw.githubusercontent.com/noir-lang/noirup/main/install | bash
noirup

# tmux
sudo apt install -y tmux

# Системные библиотеки для Rust (openssl, pkg-config)
sudo apt install -y libssl-dev pkg-config build-essential
```

### Проверка

```bash
docker --version           # Docker version 27+
rustc --version            # rustc 1.89+
node --version             # v22+
anchor --version           # anchor-cli 0.30.1
solana --version           # solana-cli 2.x
nargo --version            # nargo 1.0.0-rc.1
tmux -V                    # tmux 3.5+
```

---

## Шаг 1: Клонирование

```bash
git clone https://github.com/kwebhub/private-transfer.git
cd private-transfer
```

---

## Шаг 2: Docker-окружение

### 2.1. Клонировать Sunspot

Sunspot живёт в `workspace/sunspot` (не в git, отдельный репозиторий):

```bash
cd workspace
git clone https://github.com/reilabs/sunspot.git
cd ..
```

### 2.2. Запустить инфраструктуру

```bash
docker compose up -d
```

**Что поднимется:**

- `solana-dev` — Ubuntu с Rust, Anchor, Solana CLI, Nargo, Sunspot
- `postgres` — БД на `5432`
- `redis` — кеш на `6379`
- `prometheus` — метрики на `9090`
- `grafana` — визуализация на `3000`

### 2.3. Проверка

```bash
docker compose ps
```

Все сервисы должны быть `Up`.

---

## Шаг 3: ZK-схема (Noir + Sunspot)

**Внутри контейнера `solana-dev`:**

```bash
docker compose exec solana-dev bash
```

### 3.1. Компиляция Noir-схем

```bash
cd ~/workspace/ptrans/app/circuits/withdrawal

# Компиляция ACIR
nargo compile

# Проверка
ls target/
# → withdrawal.json
```

### 3.2. Три circuit'а

В проекте **три** circuit'а:

- `withdrawal/` — основной circuit (публичные входы: root, nullifier_hash, recipient, amount).
- `hash2/` — утилитарный (`hash_2(left, right)`).
- `hashes/` — утилитарный (`hash_1`, `hash_3`).

Каждый нужно скомпилировать:

```bash
cd ~/workspace/ptrans/app/circuits/withdrawal && nargo compile
cd ~/workspace/ptrans/app/circuits/hash2     && nargo compile
cd ~/workspace/ptrans/app/circuits/hashes    && nargo compile
```

### 3.3. Sunspot: CCS + ключи

```bash
cd ~/workspace/ptrans/app/circuits/withdrawal

# ACIR → CCS
sunspot compile target/withdrawal.json

# CCS → proving key (.pk) + verifying key (.vk)
sunspot setup target/withdrawal.ccs

# Деплой verifier на Solana
sunspot deploy target/withdrawal.vk
```

**Артефакты:**

```
target/
├── withdrawal.json           # ACIR (Noir)
├── withdrawal.ccs            # CCS
├── withdrawal.pk             # Proving key
├── withdrawal.vk             # Verifying key
├── withdrawal.so             # Compiled verifier
└── withdrawal-keypair.json   # Keypair для deploy
```

### 3.4. Получить Program ID верификатора

```bash
solana address -k target/withdrawal-keypair.json
```

Записать в `programs/ptrans/src/constants.rs`:

```rust
pub const VERIFIER_PROGRAM_ID: Pubkey = pubkey!("<твой ID>");
```

### 3.5. Скопировать circuits во frontend и merkle

```bash
# Frontend
mkdir -p ~/workspace/ptrans/app/frontend/public/circuits
cp target/withdrawal.json ~/workspace/ptrans/app/frontend/public/circuits/
cp ~/workspace/ptrans/app/circuits/hash2/target/hash2.json  ~/workspace/ptrans/app/frontend/public/circuits/
cp ~/workspace/ptrans/app/circuits/hashes/target/hashes.json ~/workspace/ptrans/app/frontend/public/circuits/

# Merkle
mkdir -p ~/workspace/ptrans/app/merkle/circuits
cp ~/workspace/ptrans/app/circuits/hash2/target/hash2.json       ~/workspace/ptrans/app/merkle/circuits/
cp ~/workspace/ptrans/app/circuits/withdrawal/target/withdrawal.json ~/workspace/ptrans/app/merkle/circuits/
```

---

## Шаг 4: Solana program (Anchor)

**Внутри контейнера `solana-dev`:**

### 4.1. Настройка кошелька

```bash
solana config set --url devnet

# Если кошелька нет — создать
solana-keygen new

# Проверить
solana address
solana balance
# Если мало SOL — airdrop
solana airdrop 2
```

### 4.2. Сборка программы

```bash
cd ~/workspace/ptrans
anchor build
```

**Артефакты:**

```
target/deploy/
├── ptrans.so
└── ptrans-keypair.json
target/idl/ptrans.json
```

### 4.3. Деплой

```bash
anchor deploy --provider.cluster devnet
```

**Если ошибка `ExtendProgram requires a minimum of 10240 additional bytes`:**

```bash
solana program extend <PROGRAM_ID> 10240 --url devnet
anchor deploy --provider.cluster devnet
```

### 4.4. Program ID

Записать в `.env`:

```
PROGRAM_ID=<твой program_id>
```

---

## Шаг 5: База данных

### 5.1. Применить миграции

Из **хоста**:

```bash
docker compose exec -T postgres psql -U ptrans -d ptrans \
  < workspace/ptrans/app/backend/migrations/001_init.sql
```

**Что создаст:**

```sql
commitments (id, leaf_index, commitment, pool_address, tx_signature, created_at)
roots       (id, root, leaf_index, pool_address, tx_signature, created_at)
nullifiers  (nullifier_hash PK, pool_address, recipient, amount, tx_signature, used_at)
```

### 5.2. Проверка

```bash
docker compose exec postgres psql -U ptrans -d ptrans -c "\dt"
```

---

## Шаг 6: Backend

### 6.1. Конфигурация

```bash
cp .env.example workspace/ptrans/.env
# отредактировать workspace/ptrans/.env
```

**Обязательные переменные:**

```
DATABASE_URL=postgres://ptrans:ptrans_dev_password@localhost:5432/ptrans
REDIS_URL=redis://localhost:6379
PROVER_URL=http://localhost:4002
MERKLE_URL=http://localhost:4003
POOL_ADDRESS=<PDA ["pool3"] от PROGRAM_ID>
```

**Узнать POOL_ADDRESS:**

```typescript
// В браузере на странице приложения
const { findPoolPda } = await import('/src/generated/pdas/index.ts');
const [pda] = await findPoolPda();
console.log(pda.toString());
```

### 6.2. Сборка

```bash
cd workspace/ptrans/app/backend
cargo build --release
```

### 6.3. Запуск

```bash
PORT=4001 cargo run --release
```

**Ожидаемый вывод:**

```
📊 Metrics initialized
✅ Connected to Postgres
✅ Connected to Redis
🌳 Initializing empty tree in Redis...
🌳 Initialized empty tree: root=3039bcb2...
🔄 Indexer started for pool ...
🚀 Backend server running on http://0.0.0.0:4001
```

---

## Шаг 7: Merkle-сервис

### 7.1. Установка

```bash
cd workspace/ptrans/app/merkle
pnpm install
```

### 7.2. Запуск

```bash
node src/server.js
```

**Ожидаемый вывод:**

```
🌳 Merkle service running on http://0.0.0.0:4003
```

### 7.3. Проверка

```bash
curl http://localhost:4003/health
# → {"status":"ok"}

curl -X POST http://localhost:4003/hash \
  -H "Content-Type: application/json" \
  -d '{"left": "0x0", "right": "0x0"}'
# → {"hash":"18dfb8dc9b82229cff974efefc8df78b1ce96d9d844236b496785c698bc6732e"}
```

---

## Шаг 8: Prover

**Внутри контейнера `solana-dev`:**

```bash
cd ~/workspace/ptrans/app/prover
PROVER_PORT=4002 cargo run
```

**Ожидаемый вывод:**

```
🔐 Prover service running on http://0.0.0.0:4002
```

### Проверка

Из **хоста** (или другого терминала контейнера):

```bash
curl -X POST http://localhost:4002/prove \
  -H "Content-Type: application/json" \
  -d '{"witness": ""}'
# → ошибка про пустой witness — это норма
```

---

## Шаг 9: Frontend

### 9.1. Конфигурация

```bash
cp .env.example workspace/ptrans/app/frontend/.env
# отредактировать workspace/ptrans/app/frontend/.env
```

### 9.2. Установка

```bash
cd workspace/ptrans/app/frontend
pnpm install
```

### 9.3. Codama (после изменения IDL)

```bash
pnpm codama
```

**Что генерирует:** `src/generated/` — инструкции, аккаунты, PDA из `target/idl/ptrans.json`.

### 9.4. Запуск

```bash
pnpm dev
```

**Открыть:** `http://localhost:5173`

---

## Шаг 10: Мониторинг

### 10.1. Prometheus

Уже запущен в Docker. Открыть `http://localhost:9090/targets`.

**Что должно быть:**

- `prometheus (1/1 up)`
- `ptrans-backend (1/1 up)` — target `host.docker.internal:4001`

### 10.2. Grafana

Открыть `http://localhost:3000`.

- Login: `admin` / `admin`
- Дашборд: **Dashboards → ptrans → ptrans Overview**

**Панели:**

1. HTTP Requests per Second
2. HTTP Latency (p50/p95)
3. Total Deposits
4. Total Withdrawals
5. Merkle Tree Size
6. Indexer Lag
7. Merkle add_leaf Duration
8. Errors per Second
9. Internal Operations Duration

---

## Запуск всего

После первой установки — **всё запускается одной командой**:

```bash
./start-all.sh
```

**Что делает:**

1. `docker compose up -d postgres redis` — инфраструктура.
2. Проверяет `solana-dev` — если не запущен, поднимает.
3. Запускает `merkle` в tmux.
4. Запускает `prover` в tmux (внутри `solana-dev`).
5. Запускает `backend` в tmux.
6. Запускает `frontend` в tmux.
7. Ждёт готовности health-check'ов.

**Команды:**

```bash
./start-all.sh                     # запустить всё
./start-all.sh status              # статус всех сервисов
./start-all.sh stop                # остановить всё
./start-all.sh attach backend      # подключиться к tmux-сессии backend
./start-all.sh logs backend        # tail логов backend
./start-all.sh help                # справка
```

---

## Проверка end-to-end

### 1. Инициализировать pool

В браузере `http://localhost:5173`:

1. Подключить Phantom (devnet).
2. Нажать **Initialize Pool**.
3. Подтвердить в Phantom.

### 2. Сделать депозит

1. Ввести `0.01` SOL.
2. Нажать **Deposit**.
3. Сохранить note (скопировать).

### 3. Вывести

1. Вставить note.
2. Ввести адрес получателя.
3. Ввести `0.01` SOL.
4. Нажать **Withdraw**.
5. Подождать ~40 секунд (proof generation).
6. Подтвердить в Phantom.

### 4. Проверить в Explorer

Транзакция withdraw должна содержать лог:

```
Program log: "Proof verified successfully!"
```

И перевод SOL из vault на recipient.

---

## Troubleshooting

| Проблема | Решение |
| ---------- | --------- |
| `PoolTimedOut` | Postgres не запущен → `docker compose up -d postgres` |
| `Blockhash not found` | RPC не отвечает → проверить `SOLANA_RPC_URL` |
| `InvalidVerifier` | Неверный `VERIFIER_PROGRAM_ID` в `.env` |
| `InvalidRoot` | Не обновили root после депозита (нужен новый депозит) |
| `PrivilegeEscalation` | Старая версия программы → `anchor deploy` |
| `AccountAlreadyInUse` | Nullifier уже использован (double-spend) |
| `openssl-sys build failed` | `sudo apt install libssl-dev pkg-config` |
| Grafana «No data» | Проверить UID датасорса в `ptrans.json` |
| `host.docker.internal` не резолвится | Добавить `extra_hosts: host-gateway` в prometheus |
