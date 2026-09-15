# ZK Explained

> Как работает ZK-часть zk-pool: commitments, nullifiers, Merkle trees,
> Groth16-доказательства и Sunspot.

## Содержание

- [Что такое приватный перевод](#что-такое-приватный-перевод)
- [Компоненты системы](#компоненты-системы)
- [Депозит](#депозит)
- [Вывод](#вывод)
- [Почему связь теряется](#почему-связь-теряется)
- [NullifierSet vs NullifierRecord](#nullifierset-vs-nullifierrecord)
- [Почему Groth16](#почему-groth16)
- [Почему Sunspot](#почему-sunspot)

---

## Что такое приватный перевод

**Проблема:** на публичном блокчейне все транзакции видны. Если Алиса отправляет
SOL Бобу, наблюдатель видит:

- Адрес Алисы
- Адрес Боба
- Сумму
- Связь между ними

**Решение:** zk-pool разрывает связь между отправителем и получателем.

1. **Отправитель** вносит SOL в общий пул (deposit).
2. **Получатель** выводит SOL из пула (withdraw) **на любой адрес**.
3. Благодаря ZK-доказательствам **никто** не может связать депозит и вывод.

**Что видит блокчейн:**

- Commitment (хеш)
- Nullifier hash (другой хеш)
- Сумму
- Merkle root

**Что скрыто:**

- Кто именно внёс депозит
- Какой депозит был выведен
- Связь между двумя транзакциями

---

## Компоненты системы

| Компонент | Что делает | Где хранится |
| ----------- | ----------- | -------------- |
| **Commitment** | Скрывает детали депозита в хеше | В Merkle tree на Solana |
| **Nullifier** | Предотвращает двойное расходование | В NullifierRecord PDA |
| **Nullifier Hash** | Публичная версия nullifier'а | В NullifierRecord и в tx |
| **Merkle Tree** | Доказывает, что депозит в пуле | В PoolAcc и в Redis backend |
| **ZK Circuit** | Доказывает всё, не раскрывая ничего | `withdrawal.nr` (Noir) |
| **Verifier** | Проверяет Groth16-proof on-chain | Программа `Eewognja...` (Sunspot) |

### Формулы

```
commitment       = Poseidon2(nullifierSecret, secret, amount)
nullifierHash    = Poseidon2(nullifierSecret)
merkleRoot       = buildTree(commitments)
```

**Важно:** commitment и nullifierHash — **разные** хеши, но оба зависят от
`nullifierSecret`. Их невозможно связать, не зная `nullifierSecret`.

---

## Депозит

### 1. Генерация секретов (в браузере)

Когда пользователь вводит `1 SOL`:

```typescript
const nullifierSecret = crypto.getRandomValues(new Uint8Array(32)); // 32 байта
const secret          = crypto.getRandomValues(new Uint8Array(32)); // 32 байта
```

Это **приватные** данные — они **никогда** не покидают браузер.
Сохраняются в localStorage как *deposit note*.

### 2. Вычисление commitment

```typescript
const commitment = Poseidon2(nullifierSecret, secret, amount);
```

Commitment — это **публичный** хеш. Он уходит в блокчейн.

### 3. Добавление в Merkle Tree

Frontend запрашивает у backend:

- Список существующих commitments
- Новый Merkle root (с добавленным commitment)

Backend (через merkle-сервис) строит дерево и возвращает root.

### 4. Отправка транзакции

```typescript
program.deposit(pool, vault, depositor, {
  commitment,
  new_root,
  amount
});
```

**Что происходит on-chain:**

- SOL переводится с `depositor` на `vault`
- `commitment` становится новым листом дерева
- `new_root` сохраняется в `PoolAcc.roots[]`
- Эмитится `DepositEvent`

### 5. Что видит наблюдатель

```
Deposit: 0x7a3b... (commitment, хеш — не адрес!)
leaf_index: 0
amount: 1 SOL
```

Наблюдатель **не знает** `nullifierSecret` и `secret`, поэтому не может
связать commitment с конкретным пользователем.

---

## Вывод

### 1. Раскрытие nullifier hash (но не секрета)

Когда пользователь хочет вывести средства:

```typescript
const nullifierHash = Poseidon2(nullifierSecret);
```

`nullifierHash` отправляется в блокчейн **публично**. Но это **не** раскрывает
`nullifierSecret`.

### 2. Что подаётся в ZK-схему

**Публично:**

- `root` — Merkle root
- `nullifierHash` — хеш нуллификатора
- `recipient` — адрес получателя
- `amount` — сумма

**Приватно:**

- `nullifierSecret` — секрет из note
- `secret` — второй секрет из note
- `merkleProof[20]` — siblings в дереве
- `isEven[20]` — флаги сторон

### 3. Что проверяет ZK-схема

Внутри себя (не раскрывая данные блокчейну):

```noir
// 1. Восстанавливаем commitment из секретов
let commitment = Poseidon2(nullifierSecret, secret, amount);

// 2. Проверяем, что nullifierHash действительно от нашего секрета
let computed_nullifier_hash = Poseidon2(nullifierSecret);
assert(computed_nullifier_hash == nullifierHash, "Invalid nullifier hash");

// 3. Проверяем, что commitment есть в дереве с корнем root
let computed_root = compute_merkle_root(commitment, merkleProof, isEven);
assert(computed_root == root, "Invalid Merkle proof");
```

### 4. On-chain верификация

Proof отправляется в программу `ptrans`:

```rust
pub fn withdraw(ctx, proof, nullifier_hash, root, to, amount) {
    // 1. Проверяем, что root в истории
    require!(pool.is_known_root(&root), InvalidRoot);

    // 2. Проверяем ZK-proof через CPI к верификатору
    invoke(&verify_instruction, ...)?;  // ← Sunspot verifier

    // 3. Создаём NullifierRecord PDA
    //    Если PDA уже существует → Anchor падает (double-spend)

    // 4. Переводим SOL из vault
    system_program::transfer(vault → recipient, amount)?;
}
```

### 5. Что видит наблюдатель

```
Withdraw: nullifier_hash 0x9c2f... (совершенно другой хеш!)
recipient: Bob's address
amount: 1 SOL
```

**Полное отсутствие связи** с исходным депозитом.

---

## Почему связь теряется

Сравни два хеша:

```
commitment    = Hash(nullifierSecret, secret, amount)  ← 3 входа
nullifierHash = Hash(nullifierSecret)                  ← 1 вход
```

Оба используют `nullifierSecret`, но:

- **Commitment** — это хеш **от трёх** значений.
- **NullifierHash** — это хеш **от одного** значения.

**Без знания `nullifierSecret` и `secret`** невозможно доказать,
что commitment и nullifierHash связаны.

Криптографически commitment и nullifierHash выглядят как **два случайных хеша**.
Никакой статистический анализ не поможет их связать.

**Что мы знаем:**

- Что **какой-то** депозит был потрачен (nullifierHash публичный)
- Что **какой-то** депозит существует в дереве (commitment публичный)
- Что **оба** — часть одного пула

**Что мы НЕ знаем:**

- **Какой именно** депозит был потрачен
- **Кто именно** потратил
- **Связь** между депозитом и выводом

---

## NullifierSet vs NullifierRecord

### Первая версия: NullifierSet

Один аккаунт с `Vec<[u8; 32]>`:

```rust
#[account]
pub struct NullifierSetAcc {
    pub pool: Pubkey,
    #[max_len(1024)]
    pub nullifiers: Vec<[u8; 32]>,
}
```

**Проблемы:**

- **Лимит 256 элементов** — после 256 выводов пул перестаёт работать.
- **Размер 8 КБ** — rent ~0.003 SOL (не критично, но растёт).
- **Один аккаунт** — все нуллификаторы в одном месте.

### Текущая версия: NullifierRecord PDA

Отдельный PDA на каждый нуллификатор:

```rust
#[account]
pub struct NullifierRecord {
    pub pool: Pubkey,
    pub nullifier_hash: [u8; 32],
    pub recipient: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
```

Seeds: `["nullifier_record", pool, nullifier_hash]`.

**Плюсы:**

- **Неограниченное** количество выводов.
- **O(1)** проверка: если PDA существует → double-spend.
- **Изоляция:** одна запись = один аккаунт.
- **Гибкость:** можно удалять старые, перейти на Merkle tree для нуллификаторов.

### Почему не добавить nullifiers прямо в Pool?

**Ограничения на размер аккаунта:**

- Аккаунты Solana могут быть до 10 МБ.
- Но rent пропорционален размеру (~6.9 SOL/МБ в год).
- 256 нуллификаторов = ~8 КБ.
- Тысячи нуллификаторов = сотни КБ — дорого.

**Разделение ответственности:**

- **Pool** — состояние дерева (roots, next_leaf_index, total_deposits).
- **NullifierRecord** — потраченные депозиты.

Разные данные с разными паттернами доступа.

**Гибкость:**

- Можно изменить способ хранения нуллификаторов, не трогая Pool.
- В production можно использовать **Merkle tree для нуллификаторов**
  (как делает Light Protocol) — неограниченное количество при фиксированной памяти.

---

## Почему Groth16

### Требования Solana

- **Размер транзакции:** максимум 1232 байта.
- **Compute units:** максимум 1.4M (с `SetComputeUnitLimit`).

### Groth16

| Параметр | Groth16 | STARK |
| ---------- | --------- | ------- |
| Размер proof | **~256 байт** | 50–200 КБ |
| Время верификации | ~1.4M CU | Зависит от схемы |
| Trusted setup | **Нужен** | Не нужен |
| On-chain верификация | ✅ Проверено | ❌ Не влезает в tx |

**Groth16 — единственный практичный выбор для Solana:**

- Proof **компактный** (влезает в tx).
- Верификация **быстрая** (укладывается в 1.4M CU).
- **Компромисс:** нужен trusted setup.

### Trusted Setup

Groth16 требует **разового** процесса генерации ключей под структуру схемы.

**Опасность:** если кто-то узнает случайные параметры setup, он сможет подделывать доказательства.

**Решение:** **MPC ceremony** — многосторонние вычисления, где исходная случайность гарантированно уничтожается.

- **В разработке:** Sunspot автоматизирует setup (небезопасно, но удобно).
- **В production:** [Reilabs trusted-setup](https://github.com/reilabs/trusted-setup).

### Два ключа

- **Proving key (pk)** — для генерации доказательств (использует backend).
- **Verification key (vk)** — для верификации on-chain (использует verifier program, ~1 КБ).

В блокчейн загружается **только vk**.

---

## Почему Sunspot

[Sunspot](https://github.com/reilabs/sunspot) — инструмент для компиляции Noir-схем
в **Solana-верификаторы Groth16**.

### Что делает Sunspot

```bash
# 1. ACIR (Noir) → CCS
sunspot compile target/withdrawal.json

# 2. CCS → proving key + verifying key
sunspot setup target/withdrawal.ccs

# 3. vk → Solana program (verifier)
sunspot deploy target/withdrawal.vk

# 4. witness + ccs + pk → Groth16 proof
sunspot prove target/withdrawal.json target/withdrawal.gz \
              target/withdrawal.ccs target/withdrawal.pk
```

### Почему отдельная программа-верификатор

- **Размер:** compiled verifier ~100 КБ — не влезает в основную программу.
- **Переиспользование:** несколько программ могут использовать один верификатор.
- **Обновление:** можно обновить верификатор, не переразворачивая основную программу.
- **Аудит:** изолированная логика проще для аудита.

### Compute units

Верификация Groth16 на Solana требует ~1.4M CU:

- По умолчанию — 200K CU на транзакцию.
- С `SetComputeUnitLimit` — до 1.4M CU.
- Наш proof **укладывается** в лимит.

Это стандартный паттерн для ZK на Solana.
