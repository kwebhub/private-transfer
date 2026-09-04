import { sha256 } from "@noble/hashes/sha2.js";
import { bytesToHex as nobleBytesToHex, hexToBytes as nobleHexToBytes } from "@noble/hashes/utils.js";

/**
 * Генерация случайных байт заданной длины
 */
export function generateRandomBytes(length: number): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(length));
}

/**
 * Хеширование SHA-256
 */
export function hash(data: Uint8Array): Uint8Array {
  return sha256(data);
}

/**
 * Вычисление commitment = hash(nullifierSecret + secret + amount)
 */
export function computeCommitment(
  nullifierSecret: Uint8Array,
  secret: Uint8Array,
  amount: bigint,
): Uint8Array {
  const amountBytes = new Uint8Array(8);
  const view = new DataView(amountBytes.buffer);
  view.setBigUint64(0, amount, true);

  const data = new Uint8Array(nullifierSecret.length + secret.length + amountBytes.length);
  data.set(nullifierSecret, 0);
  data.set(secret, nullifierSecret.length);
  data.set(amountBytes, nullifierSecret.length + secret.length);

  return sha256(data);
}

/**
 * Вычисление nullifierHash = hash(nullifierSecret)
 */
export function computeNullifierHash(nullifierSecret: Uint8Array): Uint8Array {
  return sha256(nullifierSecret);
}

/**
 * Конвертация байт в hex строку
 */
export function bytesToHex(bytes: Uint8Array): string {
  return nobleBytesToHex(bytes);
}

/**
 * Конвертация hex строки в байты
 */
export function hexToBytes(hex: string): Uint8Array {
  return nobleHexToBytes(hex);
}

/**
 * Валидация hex строки
 */
export function isValidHex(hex: string): boolean {
  return /^[0-9a-fA-F]+$/.test(hex) && hex.length % 2 === 0;
}

/**
 * Проверка длины Uint8Array
 */
export function isBytesLength(bytes: Uint8Array, length: number): boolean {
  return bytes.length === length;
}

/**
 * Создание секретной пары (nullifierSecret, secret) для депозита
 */
export function generateSecrets(): { nullifierSecret: Uint8Array; secret: Uint8Array } {
  return {
    nullifierSecret: generateRandomBytes(32),
    secret: generateRandomBytes(32),
  };
}
