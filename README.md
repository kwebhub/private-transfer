<!-- Markdownlint-disable MD013 -->

# private-transrer

Confidential SOL transfers using Noir ZK (Zero-Knowledge Schemes) and Groth16 on-chain validation via Sunspot.

## frontend

действия с кашельком в проекте обозначены кнопкой и состоят из

- компонент WalletConnect.vue
- компосабл useWallet.ts
- стор wallet.ts

компонент WalletConnect.vue выводится на странице Home.vue, форматирует вывод адреса и баланса, определяет надписи на кнопке исходя из состояния и функции при клике на кнопку

компосабл useWallet.ts создаёт объект подключения к сети solana (клиента), в функции connect() находит кошелёк (расширение браузера), определяет и сохраняет в сторе адрес кошелька, обновляет в сторе баланс по адресу кошелька, вешает слушатель на изменение адреса кошелька, чтобы обновить стор или отключиться функцией стора и функцией кошелька. Обновляет адрес и баланс в сторе при обновлении страницы через onMount

стор wallet.ts определяет поля состояния и функци по изменению значений этих полей.

### Pool

```
┌─────────────────────────────────────────────────────────────────┐
│                        ПРОГРАММА (SOLANA)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │   POOL PDA   │  │  NULLIFIER   │  │     VAULT PDA        │ │
│  │              │  │    SET PDA   │  │                      │ │
│  │ • authority  │  │ • pool ref   │  │   (хранит SOL)       │ │
│  │ • nextLeaf   │  │ • nullifiers │  │                      │ │
│  │ • totalDep   │  │   [hash...]  │  │                      │ │
│  │ • roots[10]  │  └──────────────┘  └──────────────────────┘ │
│  └──────────────┘                                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

Три основных аккаунта пула:

Pool Account (`poolAcc.ts`) хранит основное состояние Merkle Tree, размер: 384 байта (фиксированный)

```typescript
{
  authority: Address,          // Владелец пула (кто инициализировал)
  nextLeafIndex: bigint,       // Следующий индекс для вставки (0, 1, 2...)
  totalDeposits: bigint,       // Общая сумма всех депозитов
  currentRootIndex: bigint,    // Индекс текущего корня (0-9)
  roots: Array<Uint8Array>     // История из 10 последних корней
}
```

Nullifier Set Account (`nullifierSetAcc.ts`), хранит использованные nullifier'ы для защиты от двойных трат:
При выводе (withdraw) проверяется, что `nullifierHash` отсутствует в этом списке.

```typescript
{
  pool: Address,               // Ссылка на пул
  nullifiers: Array<Uint8Array> // Массив хешей использованных депозитов
}
```

Vault Account (`poolVaultPda`), простой аккаунт, который хранит SOL-токены всех депозитов. Никакой структуры данных, просто баланс.

Инициализация (`pool.ts` + `init-pool.ts`)

```typescript
// Инструкция создает 3 аккаунта:
getPoolInstructionAsync({
  authority: signer  // Кто будет владельцем пула
})

// Создаются:
// 1. Pool PDA (seed = "pool")
// 2. Nullifier Set PDA (seed = "nullifier" + pool)
// 3. Vault PDA (seed = "vault" + pool)
```

**После инициализации:**

- `nextLeafIndex = 0`
- `totalDeposits = 0`
- `roots = [32 нулевых байта, ...]`
- `nullifiers = []`

### Deposit

Депозит (`deposit.ts` + `useDeposit.ts`)

```typescript
// 1. Генерация секретов
const { nullifierSecret, secret } = generateSecrets();

// 2. Вычисление commitment = hash(nullifierSecret + secret + amount)
const commitment = computeCommitment(nullifierSecret, secret, amount);

// 3. Вычисление nullifierHash = hash(nullifierSecret)
const nullifierHash = computeNullifierHash(nullifierSecret);

// 4. Расчет нового корня (с учетом пустых листьев)
const targetNewRoot = calculateNextMerkleRoot(nextLeafIndex, commitment);

// 5. Отправка транзакции с инструкцией deposit
//    - Добавляет commitment как новый лист в Merkle Tree
//    - Переводит SOL в vault
//    - Обновляет nextLeafIndex, totalDeposits, roots
```

**Расчет корня:**

```
Уровень 0: [leaf0] [leaf1] [leaf2] [leaf3] ... (листья - commitment'ы)
Уровень 1: [hash(leaf0+leaf1)] [hash(leaf2+leaf3)] ...
Уровень 2: [hash(hash0+hash1)] [hash(hash2+hash3)] ...
...
Уровень 20: [ROOT] (один корень)
```

**Пустые листья:** Всегда `32 нулевых байта`.

```
┌──────────────────────────────────────────────────────────┐
│                   FRONTEND (Vue 3)                      │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  DepositForm.vue                                       │
│       ↓                                                 │
│  usePool.ts ─────────► fetchPoolInfo()                 │
│       ↓                     ↓                           │
│  useDeposit.ts ─────────► deposit()                    │
│       ↓                     ↓                           │
│  crypto.ts ──────────────► generateSecrets()           │
│       ↓                     ↓                           │
│  deposits.ts (store) ───► addNote()                   │
│                                                          │
└──────────────────────────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────┐
│              @solana/kit (RPC клиент)                   │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  getProgramAccounts ───► поиск аккаунтов               │
│  fetchPoolAcc ─────────► чтение PoolAcc                │
│  sendTransaction ──────► отправка транзакций          │
│                                                          │
└──────────────────────────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────┐
│              SOLANA DEVNET                              │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  Program: FbXJSZ171dcnHJVrd5E6KwvXAx7bMgxC44McF4vJ6cK │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

Безопасность и проверки

| Что проверяется | Где | Как |
| ---------------- | ----- | ----- |
| Минимальный депозит | `useDeposit.ts` | `amountSol < MIN_DEPOSIT` |
| Подпись депозитора | Программа | Проверка подписи транзакции |
| Корректность корня | Программа | `newRoot` должен совпадать с вычисленным |
| Двойная трата | Программа | Проверка `nullifierHash` в Nullifier Set |
| Переполнение дерева | Программа | `nextLeafIndex < 2^20` |
| Баланс Vault | Программа | При выводе проверяется достаточность |

1. **Подключи кошелек** (Phantom/Solflare)
2. **Если пул не создан** → нажми "Initialize Pool"
3. **После создания** → введи сумму и нажми "Deposit"
4. **Сохрани Deposit Note** (она понадобится для вывода)

Nullifier Secret: 966d7f21ff0d2adb3c998bc83ae98c53c7d56f3008517d616a2a3b95ad5eb334
Secret: e86d6e32ba27150176c58b8151eae0baf33049f0fc1317321326464ddd8e94ca
Amount: 0.01 SOL
Commitment: 2b90f2a34b8b39784aa4931927c43cb92ba88925cb08829317692002424aadfe

### withdraw

- Пользователь предоставляет `nullifierSecret` и `secret`
- Генерируется ZK-доказательство
- Проверяется `nullifierHash` в Nullifier Set
- Средства выводятся из Vault
- `nullifierHash` добавляется в Nullifier Set

# 📊 Полный анализ процесса снятия депозита (Withdraw)

Я проанализировал все файлы. Вот как работает withdraw от начала до конца.

---

## 🏗️ Общая схема

```
┌──────────────────────────────────────────────────────────────────────┐
│ 1. FRONTEND (useWithdraw.ts)                                        │
│    • Юзер вводит Deposit Note + recipient + amount                  │
│    • Вычисляет nullifierHash локально                               │
│    • Проверяет, что нота не использована                            │
│    • Отправляет запрос на бэкенд: POST /api/withdraw                │
└──────────────────────────────────────────────────────────────────────┘
                              ↓
┌──────────────────────────────────────────────────────────────────────┐
│ 2. BACKEND (main.rs + merkle_tree.rs)                               │
│    • Получает: nullifierSecret, secret, amount, recipient            │
│    • Строит Merkle proof для commitment                             │
│    • Генерирует ZK-доказательство (Gnark, 324 байта)               │
│    • Возвращает: proof, nullifierHash, root                         │
└──────────────────────────────────────────────────────────────────────┘
                              ↓
┌──────────────────────────────────────────────────────────────────────┐
│ 3. FRONTEND                                                          │
│    • Создает withdrawInstruction через Codama                       │
│    • Отправляет транзакцию через Wallet Standard                    │
└──────────────────────────────────────────────────────────────────────┘
                              ↓
┌──────────────────────────────────────────────────────────────────────┐
│ 4. SOLANA PROGRAM (withdraw.rs)                                     │
│    • Проверяет root в истории пула                                  │
│    • Проверяет nullifier не использован                             │
│    • Вызывает verifier program (CPI) с ZK-proof                     │
│    • Добавляет nullifier в set                                      │
│    • Переводит SOL из vault → recipient                             │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 🔐 Шаг 1: Что доказывает ZK-схема (Noir)

Файл `app/circuits/withdrawal/src/main.nr`:

### Публичные входы (видны всем)

```noir
root: pub Field             // корень Merkle Tree
nullifier_hash: pub Field   // хеш нуллифаера
recipient: pub Field        // получатель
amount: pub Field           // сумма
```

### Приватные входы (только у пользователя)

```noir
nullifier: Field            // секрет из ноты
secret: Field               // секрет из ноты
merkle_proof: [Field; 20]   // путь в дереве
is_even: [bool; 20]         // с какой стороны sibling
```

### Что доказывается

```noir
1. commitment = hash_3([nullifier, secret, amount])   // воссоздаем leaf
2. computed_nullifier_hash = hash_1([nullifier])      // проверяем nullifier
3. assert(computed_nullifier_hash == nullifier_hash)  // совпадает с публичным?
4. computed_root = compute_merkle_root(commitment, merkle_proof, is_even)
5. assert(computed_root == root)                      // leaf действительно в дереве
```

**Смысл:** пользователь доказывает *"я знаю секреты, которые дают commitment, лежащий в дереве с корнем `root`, и мой nullifierHash совпадает"* — **не раскрывая** `nullifier`, `secret` и путь в дереве.

---

## 🧮 Шаг 2: Бэкенд генерирует proof (`main.rs` + `merkle_tree.rs`)

### Что делает бэкенд

1. **Получает от фронта:**

   ```json
   {
     "nullifierSecret": "0x...",
     "secret": "0x...",
     "amount": 1000000000,
     "recipient": "7xKX..."
   }
   ```

2. **Строит Merkle proof** через `merkle_tree.rs`:
   - Находит `commitment = hash_3(nullifier, secret, amount)`
   - Находит индекс листа в дереве
   - Собирает путь из 20 sibling-хешей + флаги `is_even`

3. **Генерирует ZK-proof** (через Gnark, скорее всего Go-сервис рядом, хотя в коде это не видно). В `main.rs` эндпоинта `/api/withdraw` **вообще нет** — только `/api/deposit`, `/api/proof`, `/api/root`, `/api/health`.

   ⚠️ **Важное расхождение:** `useWithdraw.ts` вызывает `POST /api/withdraw`, а в `main.rs` такого роута **нет**. Значит:
   - Либо бэкенд не дописан,
   - Либо это отдельный сервис (например, Go-сервер для генерации proof),
   - Либо эндпоинт планируется добавить.

4. **Возвращает:**

   ```json
   {
     "proof": "base64...",     // ZK-доказательство (324 байта)
     "nullifierHash": "0x...",
     "root": "0x..."
   }
   ```

### Формат proof (Gnark)

- 324 байта = 4 × 32 (public inputs) + ~192 (G1/G2 точки BN254) + заголовки
- Кодируется в base64 для передачи по HTTP

---

## 🖥️ Шаг 3: Фронт вызывает withdraw (`useWithdraw.ts`)

### Пошагово

```typescript
1. Проверяет, что кошелек подключен
2. amountLamports = amountSol * 1e9

3. Вычисляет nullifierHash = hash(nullifierSecret)
   nullifierHashHex = bytesToHex(nullifierHash)

4. Ищет ноту в localStorage по nullifierHashHex
   if (!note) throw "Deposit note not found"
   if (note.used) throw "Already withdrawn"

5. Отправляет на бэкенд:
   await apiWithdraw({ nullifierSecret, secret, amount, recipient })
   → получает { proof, nullifierHash, root }

6. Находит PDA:
   - poolPda = findPoolPda()
   - vaultPda = findPoolVaultPda({ pool: poolPda })
   - nullifierSetPda = findNullifierSetPda({ pool: poolPda })

7. Декодирует proof из base64 в Uint8Array

8. Создает инструкцию через Codama:
   getWithdrawInstructionAsync({
     pool, nullifierSet, poolVault,
     recipient,               // ← здесь recipient
     verifierProgram,         // ← из .env
     proof, nullifierHash, root,
     to: recipientPubkey,     // ← дубль recipient (см. ниже)
     amount
   })

9. Подписывает через Wallet Standard
   const [signedTx] = await signProperty.signTransaction([transactionMessage])

10. Отправляет через RPC

11. Помечает ноту как used:
    depositsStore.markUsed(nullifierHash)

12. Обновляет баланс
```

---

## ⛓️ Шаг 4: Solana-программа (`withdraw.rs`)

### Accounts

| Аккаунт | Seeds | Роль |
| --------- | ------- | ------ |
| `pool` | `["pool"]` | PoolAcc — хранит roots |
| `nullifier_set` | `["nullifier", pool]` | Список использованных nullifier'ов |
| `pool_vault` | `["vault", pool]` | Хранит SOL |
| `recipient` | — | Кто получит SOL |
| `verifier_program` | — | ZK-верификатор (CPI) |
| `system_program` | — | Для transfer |

### Логика `handler_withdraw`

```rust
1. require!(recipient.key() == to)                    // сверка адреса
2. require!(pool_vault.lamports() >= amount)          // хватает ли денег
3. require!(pool.is_known_root(&root))                // корень в истории?
4. require!(!nullifier_set.contains(&nullifier_hash)) // не использован?

5. public_inputs = encode_public_inputs(root, nullifier_hash, to, amount)
   // 12-байтный Gnark-заголовок + 4 × 32 байта

6. invoke(verifier_program, [proof || public_inputs])
   // CPI к верификатору: если proof неверный → InvalidProof

7. nullifier_set.add(nullifier_hash)                  // помечаем использованным

8. system_program::transfer(vault → recipient, amount)
   // подпись через PDA seeds ["vault", pool, bump]

9. emit!(WithdrawEvent { nullifier_hash, recipient, timestamp })
```

---

## 🔗 Как связаны компоненты

```
Noir circuit (main.nr)
    ↓ компилируется в
Verifier Program (VERIFIER_PROGRAM_ID = Eewognja...)
    ↓ вызывается через
ptrans::withdraw (withdraw.rs)
    ↑ инструкция создается
Codama-generated withdraw.ts
    ↑ импортируется
useWithdraw.ts
    ↑ отправляет запрос
Backend /api/withdraw (не реализован в main.rs!)
    ↑ запрос от
WithdrawForm.vue
```

---

## ⚠️ Проблемы, которые я вижу

### 1. Бэкенд не реализует `/api/withdraw`

В `main.rs` есть `/api/deposit`, `/api/proof`, `/api/root`, `/api/health` — но **нет** `/api/withdraw`. Фронт его вызывает, значит:

- Либо бэкенд не дописан,
- Либо proof генерируется в другом сервисе (Go/Gnark),
- Либо эндпоинт нужно добавить.

### 2. Дублирование `recipient` и `to`

В `getWithdrawInstructionAsync` передаются **и** `recipient`, **и** `to`. Судя по IDL, в data инструкции есть поле `to: Address`, а в accounts — `recipient`. Оба нужны, потому что программа сверяет их:

```rust
require!(ctx.accounts.recipient.key() == to, PtransError::RecipientMismatch);
```

### 3. `merkle_tree.rs` перестраивает все дерево при каждом добавлении

```rust
async fn rebuild_tree(&mut self) -> Result<...> {
    let mut leaves = vec![[0u8; 32]; 1 << TREE_DEPTH]; // 1M листьев!
    // ...
}
```

Это **O(2^20)** на каждый депозит — очень медленно и не масштабируется. Плюс нет персистентности — при рестарте бэкенда все данные теряются (в `main.rs` видно `Pool::new()`, который создает пустой пул).

### 4. `pool.is_known_root()` использует линейный поиск

```rust
self.roots.iter().any(|r| r == root)
```

10 элементов — не критично, но всё же.

### 5. NullifierSet хранит все nullifier'ы в одном аккаунте

```rust
pub nullifiers: Vec<[u8; 32]>,  // max 1024
```

- Ограничение **1024 nullifier'а** → после 1024 выводов пул перестанет работать.
- Аккаунт растет, дорожает.

### 6. `nullifier_hash` считается **дважды**

- Фронт: `computeNullifierHash(nullifierSecret)` — SHA-256
- Программа: verifier проверяет `hash_1([nullifier])` через Poseidon

⚠️ **Это разные хеши!** Фронт использует SHA-256, а схема Noir — Poseidon. Значит, `nullifierHash`, отправленный в программу, **не совпадет** с тем, что проверяет ZK-верификатор.

Скорее всего, это баг: либо фронт должен использовать Poseidon-хеш, либо бэкенд должен возвращать правильный `nullifierHash`. В `useWithdraw.ts` `nullifierHash` берется **из ответа бэкенда** для отправки в транзакцию, а локальный SHA-256 используется только для поиска ноты. Это может быть намеренно, но выглядит хрупко.

### 7. `encode_public_inputs` в Rust vs Noir

- Rust: `amount` упакован в 32 байта (big-endian, справа)
- Noir: `amount: pub Field`

Нужно убедиться, что Gnark-верификатор ожидает именно такой формат (12-байтный заголовок + 4 × 32).

---

## 🎯 Что дальше?

Мы разобрали withdraw. Теперь, как ты сказал, переходим к **PostgreSQL + Redis** для бэкенда.

Судя по текущему состоянию:

- `db.rs` уже есть в проекте, но я его не видел.
- `config.rs` тоже не видел.

**Что мне нужно от тебя, чтобы начать работу с БД:**

1. `app/backend/src/db.rs` — текущая реализация (если есть)
2. `app/backend/src/config.rs` — конфиг
3. `app/backend/Cargo.toml` — зависимости (есть ли `sqlx`, `redis`, `deadpool` и т.д.)
4. Есть ли уже docker-compose / миграции?
5. Что именно нужно хранить в PostgreSQL, а что в Redis?

Скидывай — и поехали дальше.
