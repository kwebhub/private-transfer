import Fastify from "fastify";
import cors from "@fastify/cors";
import { Noir } from "@noir-lang/noir_js";
import { readFileSync } from "fs";
import { fileURLToPath } from "url";
import { dirname, join } from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const TREE_DEPTH = 20;

// Загружаем circuit один раз при старте
const hash2Circuit = JSON.parse(readFileSync(join(__dirname, "../circuits/hash2.json"), "utf8"));
const hash2Noir = new Noir(hash2Circuit);

// ============================================================
// Poseidon2 hash от двух 32-байтных hex-строк
// ============================================================
async function poseidon2Hash(leftHex, rightHex) {
  const { returnValue } = await hash2Noir.execute({
    left: leftHex.startsWith("0x") ? leftHex : "0x" + leftHex,
    right: rightHex.startsWith("0x") ? rightHex : "0x" + rightHex,
  });
  // returnValue — hex-строка "0x..."
  const clean = returnValue.startsWith("0x") ? returnValue.slice(2) : returnValue;
  return clean.padStart(64, "0");
}

// ============================================================
// Построение дерева из commitments
// Возвращает { root, tree } где tree[0] — листья, tree[DEPTH] — корень
// ============================================================
async function buildTree(commitments) {
  const totalLeaves = 1 << TREE_DEPTH;
  const emptyLeaf = "0".repeat(64);

  let level = new Array(totalLeaves);
  for (let i = 0; i < totalLeaves; i++) {
    level[i] = i < commitments.length ? commitments[i].padStart(64, "0") : emptyLeaf;
  }

  const tree = [level];

  for (let d = 0; d < TREE_DEPTH; d++) {
    const nextLen = level.length / 2;
    const next = new Array(nextLen);
    for (let i = 0; i < nextLen; i++) {
      next[i] = await poseidon2Hash(level[2 * i], level[2 * i + 1]);
    }
    tree.push(next);
    level = next;
  }

  return tree;
}

// ============================================================
// Сервер
// ============================================================
const app = Fastify({ logger: true });
await app.register(cors, { origin: true });

// POST /root — корень по списку commitments
app.post("/root", async (request, reply) => {
  const { commitments } = request.body;
  if (!Array.isArray(commitments)) {
    return reply.code(400).send({ error: "commitments must be an array" });
  }

  const tree = await buildTree(commitments);
  const root = tree[tree.length - 1][0];

  return { root };
});

// POST /proof — Merkle proof для leaf_index
app.post("/proof", async (request, reply) => {
  const { commitments, leaf_index } = request.body;

  if (!Array.isArray(commitments)) {
    return reply.code(400).send({ error: "commitments must be an array" });
  }
  if (typeof leaf_index !== "number" || leaf_index < 0 || leaf_index >= commitments.length) {
    return reply.code(400).send({ error: "leaf_index out of range" });
  }

  const tree = await buildTree(commitments);

  const proof = [];
  const isEven = [];
  let idx = leaf_index;

  for (let d = 0; d < TREE_DEPTH; d++) {
    const siblingIdx = idx % 2 === 0 ? idx + 1 : idx - 1;
    proof.push(tree[d][siblingIdx]);
    isEven.push(idx % 2 === 0);
    idx = Math.floor(idx / 2);
  }

  const root = tree[tree.length - 1][0];

  return { proof, is_even: isEven, root };
});

// GET /health
app.get("/health", async () => ({ status: "ok" }));

// ============================================================
// Старт
// ============================================================
const port = Number(process.env.MERKLE_PORT || 4003);
await app.listen({ port, host: "0.0.0.0" });
console.log(`🌳 Merkle service running on http://0.0.0.0:${port}`);
