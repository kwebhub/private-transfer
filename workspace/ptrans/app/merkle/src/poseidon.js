import { Noir } from "@noir-lang/noir_js";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Загружаем circuit один раз
const hash2Circuit = JSON.parse(readFileSync(join(__dirname, "../circuits/hash2.json"), "utf8"));

// Один инстанс Noir — переиспользуется
const hash2Noir = new Noir(hash2Circuit);

/**
 * Poseidon2 hash двух 32-байтных hex-строк.
 * @param {string} leftHex
 * @param {string} rightHex
 * @returns {Promise<string>} 64 hex-символа, lowercase
 */
export async function poseidon2Hash(leftHex, rightHex) {
  const { returnValue } = await hash2Noir.execute({
    left: leftHex.startsWith("0x") ? leftHex : "0x" + leftHex,
    right: rightHex.startsWith("0x") ? rightHex : "0x" + rightHex,
  });
  const clean = returnValue.startsWith("0x") ? returnValue.slice(2) : returnValue;
  return clean.padStart(64, "0");
}

/**
 * Строит Merkle Tree из commitments.
 * @param {string[]} commitments — hex-строки (без 0x)
 * @param {number} depth
 * @returns {Promise<{tree: string[][], root: string}>}
 */
export async function buildTree(commitments, depth = 20) {
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

  return { tree, root: tree[tree.length - 1][0] };
}

/**
 * Строит Merkle proof для leaf_index.
 */
export async function buildProof(commitments, leafIndex, depth = 20) {
  const { tree, root } = await buildTree(commitments, depth);

  const proof = [];
  const isEven = [];
  let idx = leafIndex;

  for (let d = 0; d < depth; d++) {
    const siblingIdx = idx % 2 === 0 ? idx + 1 : idx - 1;
    proof.push(tree[d][siblingIdx]);
    isEven.push(idx % 2 === 0);
    idx = Math.floor(idx / 2);
  }

  return { proof, isEven, root };
}
