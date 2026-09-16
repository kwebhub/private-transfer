import Fastify from "fastify";
import cors from "@fastify/cors";
import { poseidon2Hash, buildTree, buildProof } from "./poseidon.js";

const TREE_DEPTH = 20;

// ============================================================
// Сервер
// ============================================================
const app = Fastify({ logger: true });
await app.register(cors, { origin: true });

// POST /hash — Poseidon2 hash двух элементов
app.post("/hash", async (request, reply) => {
  const { left, right } = request.body;

  if (typeof left !== "string" || typeof right !== "string") {
    return reply.code(400).send({ error: "left and right must be hex strings" });
  }

  const hash = await poseidon2Hash(left, right);
  return { hash };
});

// POST /root — корень по списку commitments
app.post("/root", async (request, reply) => {
  const { commitments } = request.body;
  if (!Array.isArray(commitments)) {
    return reply.code(400).send({ error: "commitments must be an array" });
  }

  const { root } = await buildTree(commitments, TREE_DEPTH);
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

  const { proof, isEven, root } = await buildProof(commitments, leaf_index, TREE_DEPTH);
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
