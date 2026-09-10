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
