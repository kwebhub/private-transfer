import { Noir } from "@noir-lang/noir_js";

// ============ Загрузка circuit'ов ============
let hashesCircuitPromise: Promise<any> | null = null;
let hash2CircuitPromise: Promise<any> | null = null;
let withdrawalCircuitPromise: Promise<any> | null = null;

async function loadHashesCircuit(): Promise<any> {
  if (!hashesCircuitPromise) {
    hashesCircuitPromise = fetch("/circuits/hashes.json").then((r) => {
      if (!r.ok) throw new Error(`Failed to load hashes.json: ${r.status}`);
      return r.json();
    });
  }
  return hashesCircuitPromise;
}

async function loadHash2Circuit(): Promise<any> {
  if (!hash2CircuitPromise) {
    hash2CircuitPromise = fetch("/circuits/hash2.json").then((r) => {
      if (!r.ok) throw new Error(`Failed to load hash2.json: ${r.status}`);
      return r.json();
    });
  }
  return hash2CircuitPromise;
}

async function loadWithdrawalCircuit(): Promise<any> {
  if (!withdrawalCircuitPromise) {
    withdrawalCircuitPromise = fetch("/circuits/withdrawal.json").then((r) => {
      if (!r.ok) throw new Error(`Failed to load withdrawal.json: ${r.status}`);
      return r.json();
    });
  }
  return withdrawalCircuitPromise;
}

// ============ Утилиты конвертации ============

/**
 * Конвертация Uint8Array (32 байта, big-endian) → hex-строка "0x..."
 */
function bytesToFieldHex(bytes: Uint8Array): string {
  return "0x" + Buffer.from(bytes).toString("hex");
}

/**
 * Конвертация hex-строка "0x..." → Uint8Array (32 байта)
 * Если байт меньше 32 — дополняется нулями слева
 */
function fieldHexToBytes(hex: string): Uint8Array {
  const clean = hex.startsWith("0x") ? hex.slice(2) : hex;
  const padded = clean.padStart(64, "0");
  return Uint8Array.from(Buffer.from(padded, "hex"));
}

/**
 * Конвертация bigint → 32-байтный Uint8Array (big-endian)
 */
function bigintToBytes32(n: bigint): Uint8Array {
  const hex = n.toString(16).padStart(64, "0");
  return Uint8Array.from(Buffer.from(hex, "hex"));
}

// ============ Вычисление commitment и nullifier_hash ============

export interface CommitmentResult {
  commitment: Uint8Array;
  nullifierHash: Uint8Array;
}

/**
 * Вычисляет commitment и nullifier_hash через Noir-circuit `hashes.json`.
 *
 * @param nullifierSecret - 32 байта (big-endian Field)
 * @param secret - 32 байта (big-endian Field)
 * @param amount - сумма в lamports
 */
export async function computeCommitment(
  nullifierSecret: Uint8Array,
  secret: Uint8Array,
  amount: bigint,
): Promise<CommitmentResult> {
  const circuit = await loadHashesCircuit();
  const noir = new Noir(circuit);

  const inputs = {
    nullifier: bytesToFieldHex(nullifierSecret),
    secret: bytesToFieldHex(secret),
    amount: "0x" + amount.toString(16),
  };

  const { returnValue } = await noir.execute(inputs);

  // returnValue — массив из двух hex-строк: [commitment, nullifier_hash]
  const [commitmentHex, nullifierHashHex] = returnValue as [string, string];

  return {
    commitment: fieldHexToBytes(commitmentHex),
    nullifierHash: fieldHexToBytes(nullifierHashHex),
  };
}

/**
 * Вычисляет только nullifier_hash (без commitment).
 * Полезно для поиска ноты в localStorage.
 */
export async function computeNullifierHash(nullifierSecret: Uint8Array): Promise<Uint8Array> {
  const circuit = await loadHashesCircuit();
  const noir = new Noir(circuit);

  const inputs = {
    nullifier: bytesToFieldHex(nullifierSecret),
    secret: "0x0",
    amount: "0x0",
  };

  const { returnValue } = await noir.execute(inputs);
  const [, nullifierHashHex] = returnValue as [string, string];

  return fieldHexToBytes(nullifierHashHex);
}

/**
 * Poseidon2 hash от двух 32-байтных элементов.
 * Соответствует `hash_2([left, right])` в Noir:
 *   poseidon2_permutation([left, right, 0, 0])[0]
 *
 * Используется для вычисления корня Merkle Tree.
 */
export async function poseidon2Hash(left: Uint8Array, right: Uint8Array): Promise<Uint8Array> {
  const circuit = await loadHash2Circuit();
  const noir = new Noir(circuit);

  const { returnValue } = await noir.execute({
    left: bytesToFieldHex(left),
    right: bytesToFieldHex(right),
  });

  return fieldHexToBytes(returnValue as string);
}

// ============ Witness generation для withdraw ============

export interface WitnessInputs {
  root: Uint8Array;
  nullifierHash: Uint8Array;
  recipient: Uint8Array; // 32 байта Pubkey
  amount: bigint;
  nullifierSecret: Uint8Array;
  secret: Uint8Array;
  merkleProof: Uint8Array[]; // 20 × 32 байта
  isEven: boolean[]; // 20 булевых
}

/**
 * Генерирует witness для withdraw через `withdrawal.json`.
 * Возвращает gzip-сжатый witness (Uint8Array), готовый для отправки на сервер.
 */
export async function generateWithdrawalWitness(inputs: WitnessInputs): Promise<Uint8Array> {
  const circuit = await loadWithdrawalCircuit();
  const noir = new Noir(circuit);

  const executionInputs = {
    root: bytesToFieldHex(inputs.root),
    nullifier_hash: bytesToFieldHex(inputs.nullifierHash),
    recipient: bytesToFieldHex(inputs.recipient),
    amount: "0x" + inputs.amount.toString(16),
    nullifier: bytesToFieldHex(inputs.nullifierSecret),
    secret: bytesToFieldHex(inputs.secret),
    merkle_proof: inputs.merkleProof.map(bytesToFieldHex),
    is_even: inputs.isEven,
  };

  const { witness } = await noir.execute(executionInputs);

  return witness;
}

// ============ Экспорт утилит ============

export { bigintToBytes32, bytesToFieldHex, fieldHexToBytes };
