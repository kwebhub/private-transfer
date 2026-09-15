# Threat Model

> Модель угроз zk-pool: что защищено, от кого, какие инварианты, какие ограничения.

## Содержание

- [Активы](#активы)
- [Акторы угроз](#акторы-угроз)
- [Инварианты](#инварианты)
- [Сценарии атак](#сценарии-атак)
- [Что НЕ защищено](#что-не-защищено)
- [Митигации](#митигации)

---

## Активы

| Актив | Где | Критичность |
| ------- | ----- | ------------- |
| **SOL в vault** | On-chain (Vault PDA) | 🔴 Критично |
| **Deposit notes** | localStorage пользователя | 🔴 Критично |
| **Секреты** (`nullifierSecret`, `secret`) | localStorage + память браузера | 🔴 Критично |
| **Commitments** | Postgres + Merkle tree | 🟡 Средне |
| **Nullifiers** | Postgres + NullifierRecord PDA | 🟡 Средне |
| **Merkle tree** | Redis + PoolAcc | 🟡 Средне |
| **Trusted setup (toxic waste)** | Не существует (уничтожено) | 🔴 Критично |
| **Proving key (.pk)** | Backend (merkle-сервис не имеет) | 🟡 Средне |

---

## Акторы угроз

### 1. Внешний наблюдатель

**Возможности:**

- Видит все транзакции в блокчейне.
- Видит все публичные аккаунты и их балансы.
- Не имеет доступа к localStorage, backend, merkle.
- Не знает секретов.

**Цели:**

- Связать депозит и вывод (сломать приватность).
- Вычислить секреты (`nullifierSecret`, `secret`).

### 2. Злоумышленник с backend

**Возможности:**

- Полный доступ к Postgres, Redis, API.
- Видит commitments, nullifiers, roots.
- Видит witness? **НЕТ** — witness генерируется на клиенте.
- Видит proof + public_witness от prover.

**Цели:**

- Подделать proof (невозможно без toxic waste).
- Связать депозит и вывод (невозможно без witness).
- Слить/изменить commitments (возможно, но обнаружимо через блокчейн).

### 3. Злоумышленник с клиентом

**Возможности:**

- Внедрить XSS, украсть localStorage.
- Заменить `poseidon.ts` на поддельный.
- MITM между frontend и backend.

**Цели:**

- Украсть deposit notes (получить контроль над SOL).

### 4. Злоумышленник on-chain

**Возможности:**

- Отправить произвольную tx в программу `ptrans`.
- Подделать proof (невозможно).
- Подделать root (проверяется в `PoolAcc.roots[]`).

**Цели:**

- Вывести SOL без валидного депозита.
- Вывести один депозит дважды (double-spend).

### 5. Разработчик (доверенный, но)

**Возможности:**

- Изменить программу (upgrade authority).
- Изменить backend.
- Изменить frontend.

**Цели:**

- Случайная ошибка, а не злой умысел.

---

## Инварианты

Гарантии, которые **всегда** должны выполняться.

### I1: Каждый nullifier используется ровно один раз

**Где:** `withdraw.rs` → `init` для `NullifierRecord` PDA.

**Как:** seeds `["nullifier_record", pool, nullifier_hash]`. Если PDA существует — Anchor возвращает `AccountAlreadyInUse`.

**Тест:** повторный withdraw с той же note → ошибка.

### I2: SOL из vault переводятся только при валидном proof

**Где:** `withdraw.rs` → `invoke(verifier_program, ...)`.

**Как:** Groth16 proof верифицируется on-chain. Если `verify` падает — CPI возвращает ошибку.

**Тест:** withdraw с поддельным proof → ошибка.

### I3: Root должен быть в истории PoolAcc.roots[10]

**Где:** `withdraw.rs` → `require!(pool.is_known_root(&root))`.

**Как:** линейный поиск в массиве из 10 последних корней.

**Тест:** withdraw с произвольным root → `InvalidRoot`.

### I4: Recipient из tx совпадает с recipient из instruction

**Где:** `withdraw.rs` → `require!(recipient.key() == to)`.

**Как:** явная проверка. Recipient **входит** в public inputs ZK-схемы (приведён к Field), поэтому proof привязан к получателю.

**Тест:** подмена recipient в accounts → `RecipientMismatch`.

### I5: Commitment в Merkle tree = Poseidon2(nullifierSecret, secret, amount)

**Где:** Noir circuit `withdrawal.nr`.

**Как:** `let commitment = hash_3([nullifier, secret, amount])`. Если commitment не совпадает с листом в дереве — Merkle proof не сойдётся.

**Тест:** изменить amount → `Invalid Merkle proof`.

### I6: NullifierHash = Poseidon2(nullifierSecret)

**Где:** Noir circuit `withdrawal.nr`.

**Как:** `assert(computed_nullifier_hash == nullifier_hash)`.

**Тест:** подменить nullifier_hash → `Invalid nullifier hash`.

### I7: Vault баланс >= amount для withdraw

**Где:** `withdraw.rs` → `require!(vault.lamports() >= amount)`.

**Как:** явная проверка перед transfer.

**Тест:** withdraw больше, чем в vault → `InsufficientVaultBalance`.

---

## Сценарии атак

### A1: Double-spend

**Цель:** вывести один депозит дважды.

**Сценарий:**

1. Атакующий делает депозит → получает commitment и note.
2. Делает withdraw → NullifierRecord создан.
3. Пытается сделать withdraw **повторно** с той же note.

**Защита:** `init` для NullifierRecord → `AccountAlreadyInUse`.

**Результат:** ✅ Защищено.

**Тест:** `programs/ptrans/tests/test_ptrans.rs::test_double_spend`.

---

### A2: Фальшивый proof

**Цель:** вывести SOL без знания секретов.

**Сценарий:**

1. Атакующий генерирует произвольный proof (не соответствующий ни одному депозиту).
2. Отправляет withdraw.

**Защита:** on-chain verifier (Groth16) → `InvalidProof`.

**Результат:** ✅ Защищено (при условии, что toxic waste уничтожен).

**Тест:** withdraw с proof = `[0; 324]` → ошибка.

---

### A3: Устаревший root

**Цель:** использовать proof, сгенерированный для старого дерева.

**Сценарий:**

1. После депозита A, атакующий генерирует proof для корня `R1`.
2. Другие пользователи делают депозиты → корень меняется на `R2`.
3. Атакующий отправляет proof с `R1`.

**Защита:** `PoolAcc.roots[10]` хранит **10 последних** корней. Если `R1` ещё в истории — proof **валиден**. Если выпал — `InvalidRoot`.

**Результат:** ✅ Защищено (в пределах 10 последних корней).

**Ограничение:** если в пуле **больше 10** депозитов после proof → proof устареет.

**Митигация:** пересчитать proof перед отправкой.

---

### A4: Подмена recipient

**Цель:** направить вывод на свой адрес, используя чужой proof.

**Сценарий:**

1. Атакующий перехватывает чужой proof + public inputs.
2. Пытается отправить withdraw с recipient = свой адрес.

**Защита:** recipient входит в public inputs ZK-схемы. Если recipient в tx ≠ recipient в proof → `InvalidProof`.

**Результат:** ✅ Защищено.

---

### A5: Перехват deposit note

**Цель:** украсть deposit note и вывести SOL.

**Сценарий:**

1. Атакующий внедряет XSS на frontend.
2. Читает localStorage.
3. Получает `nullifierSecret`, `secret`.
4. Делает withdraw.

**Защита:** ❌ **НЕ защищено.**

**Митигации:**

- CSP (Content Security Policy).
- SRI (Subresource Integrity).
- Хранить notes не в localStorage, а в зашифрованном виде (пароль пользователя).

---

### A6: MITM между frontend и backend

**Цель:** подменить commitments или proof.

**Сценарий:**

1. Атакующий в той же сети перехватывает HTTP-трафик.
2. Подменяет ответ `/api/proof` — возвращает proof для другого leaf.
3. Frontend генерирует witness для неправильного commitment.
4. On-chain: proof не сойдётся → ошибка.

**Защита:** ✅ **Частично.** Если атакующий подменит proof — on-chain проверка упадёт. Но **availability** атакует (DoS).

**Митигации:**

- HTTPS (TLS) в production.
- Публичный Merkle root on-chain — frontend может **сверить** root с on-chain.

---

### A7: Атака на backend (DoS)

**Цель:** положить backend → withdraw не работает.

**Сценарий:**

1. Атакующий флудит `/api/withdraw` (тяжёлая операция).
2. Backend перегружен, легитимные запросы не проходят.

**Защита:** rate limiting (5 req/min на IP для `/api/withdraw`).

**Результат:** ✅ Защищено в базовом виде.

**Митигации для production:**

- Распределённый rate limiting (Cloudflare, Redis cluster).
- Очередь на proof generation.
- Circuit breaker для prover.

---

### A8: Атака на indexer

**Цель:** рассинхронизировать backend с блокчейном.

**Сценарий:**

1. Атакующий создаёт **много** депозитов подряд.
2. Indexer не успевает обрабатывать.
3. Backend отдаёт устаревшее дерево.

**Защита:** ✅ Частично — пагинация + `last_signature` в Redis.

**Ограничение:** при очень большом потоке (1000+ tx за 5 сек) indexer может отставать.

**Митигации:**

- Увеличить `PAGE_SIZE` и `MAX_PAGES`.
- Webhook от Helius вместо polling.
- Несколько воркеров.

---

### A9: Подмена Merkle proof на backend

**Цель:** заставить frontend сгенерировать proof для чужого commitment.

**Сценарий:**

1. Атакующий с доступом к backend модифицирует `tree.rs`.
2. Backend возвращает **неправильный** Merkle proof.
3. Frontend генерирует witness.
4. On-chain: proof не сойдётся → ошибка.

**Защита:** ✅ On-chain. Backend **не может** подделать proof — Noir-circuit + verifier проверят.

**Ограничение:** **availability**. Backend может отвечать мусором, и withdraw не работает.

---

### A10: Toxic waste в trusted setup

**Цель:** подделывать proof.

**Сценарий:**

1. При setup Groth16 остаётся случайность (toxic waste).
2. Если разработчик её сохранил — он может генерировать валидные proof без witness.

**Защита:** ❌ **НЕ защищено** в текущей версии — Sunspot использует стандартный setup.

**Митигация:** [Reilabs trusted-setup](https://github.com/reilabs/trusted-setup) — MPC ceremony, где toxic waste уничтожается.

**Статус:** TODO перед mainnet.

---

### A11: Upgrade программы

**Цель:** изменить логику программы (украсть SOL).

**Сценарий:**

1. Атакующий получает доступ к upgrade authority (`4aPAEEmJdLz4fYE44Krad3MP8Q1wAUScg3zMuC8fcBur`).
2. Загружает новую версию программы.
3. Переводит SOL из vault на свой адрес.

**Защита:** ❌ **НЕ защищено** — authority один.

**Митигации для production:**

- Мультисиг (Squads Protocol).
- Time-lock на upgrade.
- Отказ от upgrade authority после аудита.

---

## Что НЕ защищено

| Угроза | Почему | Митигация |
| -------- | -------- | ----------- |
| XSS крадёт localStorage | Все секреты в localStorage | CSP, SRI, шифрование notes |
| Backend видит commitments/nullifiers | Backend — источник истины | Не критично для приватности (связь не видна) |
| Toxic waste в setup | Sunspot стандартный setup | MPC ceremony |
| Upgrade authority один | Нет мультисига | Squads, time-lock |
| MITM меняет root | HTTP без TLS в dev | HTTPS в production |
| DoS на prover | Один сервис | Очередь, реплики |
| Indexer отстаёт | Polling | Webhook, реплики |

---

## Митигации

### Приоритет 1 (перед mainnet)

1. **MPC trusted setup** — устранить toxic waste.
2. **Аудит программы** — минимум 2 независимых аудита.
3. **Мультисиг на upgrade authority** — Squads Protocol.
4. **HTTPS** — TLS для всех эндпоинтов.
5. **CSP** — Content Security Policy для frontend.

### Приоритет 2 (после mainnet)

1. **Шифрование deposit notes** — пароль пользователя.
2. **Webhook от Helius** вместо polling.
3. **Circuit breaker** для prover/merkle.
4. **Backup Postgres** — ежедневные снапшоты.
5. **Alerting** — Grafana Alerting на аномалии.

### Приоритет 3 (nice to have)

1. **Zero-knowledge proof на клиенте** — без backend (WASM-порт Sunspot).
2. **Merkle tree для нуллификаторов** — неограниченный рост.
3. **Multi-sig для authority** — DAO-управление.
4. **Formal verification** — доказательство инвариантов.

---

## Тесты

Инварианты проверяются тестами:

| Инвариант | Тест |
| ----------- | ------ |
| I1: nullifier один раз | `test_double_spend` |
| I2: proof валиден | `test_invalid_proof` |
| I3: root в истории | `test_invalid_root` |
| I4: recipient совпадает | `test_recipient_mismatch` |
| I5: commitment совпадает | `test_invalid_commitment` |
| I6: nullifier_hash совпадает | `test_invalid_nullifier_hash` |
| I7: vault баланс | `test_insufficient_vault` |

**Статус:** TODO — написать тесты в Этапе 4.
