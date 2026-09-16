/**
 * Unit-тесты для merkle-сервиса.
 *
 * Запуск: `npm test`
 * Используется встроенный Node.js test runner (`node --test`).
 *
 * ⚠️ ВАЖНО: `buildTree` строит дерево 2^20 = 1M хешей через noir_js.
 * Это занимает минуты. Поэтому в тестах мы НЕ строим полное дерево.
 * Тестируем только `poseidon2Hash` + логику proof на маленькой глубине.
 */

import { test, describe } from "node:test";
import assert from "node:assert/strict";
import { poseidon2Hash } from "../src/poseidon.js";

const EMPTY_LEAF = "0".repeat(64);

// ============================================================
// Poseidon2 hash
// ============================================================

describe("poseidon2Hash", () => {
  test("h(0, 0) matches Noir reference", async () => {
    const hash = await poseidon2Hash(EMPTY_LEAF, EMPTY_LEAF);
    assert.equal(
      hash,
      "18dfb8dc9b82229cff974efefc8df78b1ce96d9d844236b496785c698bc6732e",
      "h(0,0) должен совпадать с Noir-эталоном",
    );
  });

  test("Poseidon2 не коммутативен: h(a,b) != h(b,a)", async () => {
    const a = "0".repeat(63) + "1";
    const b = "0".repeat(63) + "2";

    const ab = await poseidon2Hash(a, b);
    const ba = await poseidon2Hash(b, a);

    assert.notEqual(ab, ba, "Poseidon2 не должен быть коммутативным");
  });

  test("детерминированный: одинаковый вход → одинаковый выход", async () => {
    const left = "0".repeat(62) + "aa";
    const right = "0".repeat(62) + "bb";

    const h1 = await poseidon2Hash(left, right);
    const h2 = await poseidon2Hash(left, right);

    assert.equal(h1, h2);
  });

  test("возвращает ровно 64 hex-символа (32 байта)", async () => {
    const hash = await poseidon2Hash(EMPTY_LEAF, EMPTY_LEAF);
    assert.equal(hash.length, 64, "hash должен быть 32 байта (64 hex)");
    assert.match(hash, /^[0-9a-f]{64}$/, "hash должен быть lowercase hex");
  });

  test("принимает hex с префиксом 0x и без", async () => {
    const withPrefix = await poseidon2Hash("0x" + EMPTY_LEAF, "0x" + EMPTY_LEAF);
    const withoutPrefix = await poseidon2Hash(EMPTY_LEAF, EMPTY_LEAF);
    assert.equal(withPrefix, withoutPrefix);
  });
});

// ============================================================
// Merkle proof — на маленькой глубине (DEPTH=3)
// ============================================================

/**
 * Локальная функция buildProof с параметром глубины.
 * НЕ использует buildTree из poseidon.js (там depth=20 фиксирован).
 */
async function buildProofSmall(commitments, leafIndex, depth = 3) {
  const totalLeaves = 1 << depth;
  const emptyLeaf = "0".repeat(64);

  let level = new Array(totalLeaves);
  for (let i = 0; i < totalLeaves; i++) {
    level[i] = i < commitments.length ? commitments[i].padStart(64, "0") : emptyLeaf;
  }

  const tree = [level];
  for (let d = 0; d < depth; d++) {
    const nextLen = level.length / 2;
    const next = new Array(nextLen);
    for (let i = 0; i < nextLen; i++) {
      next[i] = await poseidon2Hash(level[2 * i], level[2 * i + 1]);
    }
    tree.push(next);
    level = next;
  }

  const proof = [];
  const isEven = [];
  let idx = leafIndex;

  for (let d = 0; d < depth; d++) {
    const siblingIdx = idx % 2 === 0 ? idx + 1 : idx - 1;
    proof.push(tree[d][siblingIdx]);
    isEven.push(idx % 2 === 0);
    idx = Math.floor(idx / 2);
  }

  return { proof, isEven, root: tree[tree.length - 1][0] };
}

describe("Merkle proof (depth=3)", () => {
  test("proof имеет глубину 3", async () => {
    const commitments = ["0".repeat(63) + "1"];
    const { proof, isEven } = await buildProofSmall(commitments, 0, 3);
    assert.equal(proof.length, 3);
    assert.equal(isEven.length, 3);
  });

  test("isEven[0] = true для чётного leaf_index", async () => {
    const commitments = ["0".repeat(63) + "1", "0".repeat(63) + "2"];
    const { isEven } = await buildProofSmall(commitments, 0, 3);
    assert.equal(isEven[0], true);
  });

  test("isEven[0] = false для нечётного leaf_index", async () => {
    const commitments = ["0".repeat(63) + "1", "0".repeat(63) + "2"];
    const { isEven } = await buildProofSmall(commitments, 1, 3);
    assert.equal(isEven[0], false);
  });

  test("sibling для leaf 0 — это leaf 1", async () => {
    const leaf0 = "0".repeat(63) + "1";
    const leaf1 = "0".repeat(63) + "2";
    const { proof } = await buildProofSmall([leaf0, leaf1], 0, 3);
    assert.equal(proof[0], leaf1, "sibling leaf 0 = leaf 1");
  });

  test("sibling для leaf 1 — это leaf 0", async () => {
    const leaf0 = "0".repeat(63) + "1";
    const leaf1 = "0".repeat(63) + "2";
    const { proof } = await buildProofSmall([leaf0, leaf1], 1, 3);
    assert.equal(proof[0], leaf0, "sibling leaf 1 = leaf 0");
  });
});
