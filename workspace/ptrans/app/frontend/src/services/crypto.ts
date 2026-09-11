import {
  bytesToHex as nobleBytesToHex,
  hexToBytes as nobleHexToBytes,
} from "@noble/hashes/utils.js";

/**
 * Генерация случайных байт заданной длины
 */
export function generateRandomBytes(length: number): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(length));
}

/**
 * Конвертация байт в hex строку
 */
export function bytesToHex(bytes: Uint8Array): string {
  return nobleBytesToHex(bytes as Uint8Array);
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
 * Модуль поля BN254 (scalar field of BN254 curve).
 * Используется для приведения случайных байт к валидному Field-элементу.
 */
const BN254_FIELD_MODULUS =
  21888242871839275222246405745257275088548364400416034343698204186575808495617n;

/**
 * Конвертация 32 байт (big-endian) в Field-элемент BN254 (mod p).
 * Если число >= модуля, оно автоматически уменьшается.
 */
function bytesToFieldElement(bytes: Uint8Array): Uint8Array {
  // 1. Читаем 32 байта как big-endian BigInt
  let value = 0n;
  for (let i = 0; i < 32; i++) {
    value = (value << 8n) | BigInt(bytes[i] ?? 0);
  }

  // 2. Приводим по модулю
  value = value % BN254_FIELD_MODULUS;

  // 3. Обратно в 32 байта big-endian
  const result = new Uint8Array(32);
  for (let i = 31; i >= 0; i--) {
    result[i] = Number(value & 0xffn);
    value >>= 8n;
  }

  return result;
}

/**
 * Создание секретной пары (nullifierSecret, secret) для депозита.
 * Оба значения приведены к Field-элементу BN254 (валидны для Noir).
 */
export function generateSecrets(): { nullifierSecret: Uint8Array; secret: Uint8Array } {
  return {
    nullifierSecret: bytesToFieldElement(generateRandomBytes(32)),
    secret: bytesToFieldElement(generateRandomBytes(32)),
  };
}
