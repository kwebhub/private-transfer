import { ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { useDepositsStore } from "@/stores/deposits";
import { hexToBytes, bytesToHex } from "@/services/crypto";
import {
  computeNullifierHash,
  generateWithdrawalWitness,
  poseidon2Hash,
} from "@/services/poseidon";
import { withdraw as apiWithdraw } from "@/services/api";
import {
  Connection,
  PublicKey,
  TransactionMessage,
  VersionedTransaction,
  SystemProgram,
  TransactionInstruction,
  ComputeBudgetProgram,
} from "@solana/web3.js";
import { Buffer } from "buffer";
import { findPoolPda, findPoolVaultPda, findNullifierSetPda } from "@/generated/pdas";
import { WITHDRAW_DISCRIMINATOR } from "@/generated/instructions/withdraw";
import { PTRANS_PROGRAM_ADDRESS } from "@/generated/programs";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";
const VERIFIER_PROGRAM_ID =
  import.meta.env.VITE_VERIFIER_PROGRAM_ID || "EewognjaJhZUQgP59BrCx5SaJcsFQ6sn65FEwZ2FdZhg";
const MERKLE_TREE_DEPTH = Number(import.meta.env.VITE_MERKLE_TREE_DEPTH || 20);

const BN254_MODULUS =
  21888242871839275222246405745257275088548364400416034343698204186575808495617n;

/**
 * Приводит 32 байта (big-endian) к модулю BN254.
 * Нужно для recipient Pubkey, который может превышать модуль.
 */
function reduceToField(bytes: Uint8Array): Uint8Array {
  let value = 0n;
  for (let i = 0; i < 32; i++) {
    value = (value << 8n) | BigInt(bytes[i] ?? 0);
  }
  value = value % BN254_MODULUS;
  const result = new Uint8Array(32);
  for (let i = 31; i >= 0; i--) {
    result[i] = Number(value & 0xffn);
    value >>= 8n;
  }
  return result;
}

export function useWithdraw() {
  const walletStore = useWalletStore();
  const depositsStore = useDepositsStore();

  const loading = ref(false);
  const error = ref<string | null>(null);
  const txSignature = ref<string | null>(null);
  const proofGenerating = ref(false);

  function getProvider() {
    if (typeof window === "undefined") return null;
    if ((window as any).phantom?.solana) return (window as any).phantom.solana;
    if ((window as any).solflare) return (window as any).solflare;
    if ((window as any).solana) return (window as any).solana;
    return null;
  }

  // ================================================================
  // Получение commitments из событий DepositEvent через RPC
  // ================================================================
  async function fetchCommitmentsFromChain(poolAddress: string): Promise<{
    commitments: Uint8Array[];
    latestRoot: Uint8Array;
  }> {
    const rpc = (url: string, body: unknown) =>
      fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }).then((r) => r.json());

    const sigsResp = await rpc(RPC_URL, {
      jsonrpc: "2.0",
      id: 1,
      method: "getSignaturesForAddress",
      params: [poolAddress, { limit: 1000 }],
    });

    const sigs = sigsResp.result || [];

    const entries: {
      leafIndex: number;
      commitment: Uint8Array;
      newRoot: Uint8Array;
    }[] = [];

    for (const sig of sigs) {
      const txResp = await rpc(RPC_URL, {
        jsonrpc: "2.0",
        id: 1,
        method: "getTransaction",
        params: [sig.signature, { encoding: "json", maxSupportedTransactionVersion: 0 }],
      });

      const tx = txResp.result;
      if (!tx?.meta?.logMessages) continue;

      for (const log of tx.meta.logMessages) {
        if (!log.startsWith("Program data:")) continue;
        const b64 = log.replace("Program data: ", "").trim();
        const bytes = Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));

        // DepositEvent: discriminator(8) + commitment(32) + leaf_index(8) + timestamp(8) + new_root(32)
        if (bytes.length < 8 + 32 + 8 + 8 + 32) continue;

        const commitment = bytes.slice(8, 40);
        const leafIndex = Number(new DataView(bytes.buffer).getBigUint64(40, true));
        const newRoot = bytes.slice(56, 88);

        entries.push({ leafIndex, commitment, newRoot });
      }
    }

    entries.sort((a, b) => a.leafIndex - b.leafIndex);

    if (entries.length === 0) {
      throw new Error("No deposits found for this pool");
    }

    return {
      commitments: entries.map((e) => e.commitment),
      latestRoot: entries[entries.length - 1]!.newRoot,
    };
  }

  // ================================================================
  // Построение Merkle proof для leaf_index на фронте через Noir hash2
  // ================================================================
  async function buildMerkleProof(
    commitments: Uint8Array[],
    leafIndex: number,
  ): Promise<{ proof: Uint8Array[]; isEven: boolean[] }> {
    const emptyLeaf = new Uint8Array(32);
    const cache = new Map<string, Uint8Array>();

    async function getNode(level: number, index: number): Promise<Uint8Array> {
      if (level === 0) {
        return index < commitments.length ? commitments[index]! : emptyLeaf;
      }

      const key = `${level}:${index}`;
      if (cache.has(key)) return cache.get(key)!;

      const left = await getNode(level - 1, index * 2);
      const right = await getNode(level - 1, index * 2 + 1);
      const hash = await poseidon2Hash(left, right);

      cache.set(key, hash);
      return hash;
    }

    const proof: Uint8Array[] = [];
    const isEven: boolean[] = [];

    let idx = leafIndex;
    for (let d = 0; d < MERKLE_TREE_DEPTH; d++) {
      const siblingIdx = idx % 2 === 0 ? idx + 1 : idx - 1;
      const sibling = await getNode(d, siblingIdx);
      proof.push(sibling);
      isEven.push(idx % 2 === 0);
      idx = Math.floor(idx / 2);
    }

    return { proof, isEven };
  }

  // ================================================================
  // Основная функция withdraw
  // ================================================================
  async function withdraw(
    nullifierSecretHex: string,
    secretHex: string,
    amountSol: number,
    recipientAddress: string,
  ): Promise<void> {
    loading.value = true;
    error.value = null;
    txSignature.value = null;
    proofGenerating.value = true;

    try {
      const provider = getProvider();
      if (!walletStore.isConnected || !walletStore.walletAddress || !provider) {
        throw new Error("Wallet not connected");
      }

      const amountLamports = BigInt(Math.floor(amountSol * 1_000_000_000));

      // 1. Локальный SHA-256 — только для поиска ноты в localStorage
      const nullifierSecretBytes = hexToBytes(nullifierSecretHex);
      const localNullifierHash = await computeNullifierHash(nullifierSecretBytes);
      const localNullifierHashHex = bytesToHex(localNullifierHash);

      const note = depositsStore.getNoteByNullifier(localNullifierHashHex);
      if (!note) {
        throw new Error("Deposit note not found in local storage");
      }
      if (note.used) {
        throw new Error("This deposit has already been withdrawn");
      }

      // 2. Получаем commitments и последний root из блокчейна
      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });
      const [nullifierSetPda] = await findNullifierSetPda({ pool: poolPda });

      const { commitments, latestRoot } = await fetchCommitmentsFromChain(poolPda);

      // 3. Находим leaf_index для нашего commitment'а
      const noteCommitmentHex = note.commitment.replace(/^0x/, "").toLowerCase();
      const leafIndex = commitments.findIndex(
        (c) => bytesToHex(c).toLowerCase() === noteCommitmentHex,
      );

      if (leafIndex === -1) {
        throw new Error("Your commitment not found in pool");
      }

      // 4. Строим Merkle proof
      const { proof: merkleProof, isEven } = await buildMerkleProof(commitments, leafIndex);

      // 5. Вычисляем nullifier_hash через Noir (Poseidon2)
      const nullifierHashBytes = await computeNullifierHash(nullifierSecretBytes);

      // 6. Recipient: настоящий Pubkey для транзакции + reduced для witness
      const recipientPubkey = new PublicKey(recipientAddress);
      const recipientRealBytes = recipientPubkey.toBytes();
      const recipientReducedBytes = reduceToField(recipientRealBytes);

      // 7. Генерируем witness через Noir (с reduced recipient)
      const witness = await generateWithdrawalWitness({
        root: latestRoot,
        nullifierHash: nullifierHashBytes,
        recipient: recipientReducedBytes,
        amount: amountLamports,
        nullifierSecret: nullifierSecretBytes,
        secret: hexToBytes(secretHex),
        merkleProof,
        isEven,
      });

      proofGenerating.value = false;

      // 8. Кодируем witness в base64 и отправляем на backend
      const witnessB64 = Buffer.from(witness).toString("base64");
      const withdrawResponse = await apiWithdraw({ witness: witnessB64 });

      // 9. Декодируем proof (324 байта)
      const proofBytes = Buffer.from(withdrawResponse.proof, "base64");

      // 10. Собираем data инструкции:
      //     discriminator(8) || vec<u8>(proof) || nullifier_hash(32) || root(32) || to(32) || amount(8)
      const proofOnly = proofBytes;

      const data = new Uint8Array(8 + 4 + proofOnly.length + 32 + 32 + 32 + 8);
      let offset = 0;

      data.set(WITHDRAW_DISCRIMINATOR, offset);
      offset += 8;

      new DataView(data.buffer).setUint32(offset, proofOnly.length, true);
      offset += 4;

      data.set(proofOnly, offset);
      offset += proofOnly.length;

      data.set(nullifierHashBytes, offset);
      offset += 32;

      data.set(latestRoot, offset);
      offset += 32;

      data.set(recipientRealBytes, offset);
      offset += 32;

      new DataView(data.buffer).setBigUint64(offset, amountLamports, true);

      // 11. Создаём транзакцию
      const connection = new Connection(RPC_URL, "confirmed");
      const userPublicKey = new PublicKey(walletStore.walletAddress);

      const withdrawInstruction = new TransactionInstruction({
        programId: new PublicKey(PTRANS_PROGRAM_ADDRESS),
        keys: [
          { pubkey: new PublicKey(poolPda), isSigner: false, isWritable: true },
          {
            pubkey: new PublicKey(nullifierSetPda),
            isSigner: false,
            isWritable: true,
          },
          { pubkey: new PublicKey(vaultPda), isSigner: false, isWritable: true },
          { pubkey: recipientPubkey, isSigner: false, isWritable: true },
          {
            pubkey: new PublicKey(VERIFIER_PROGRAM_ID),
            isSigner: false,
            isWritable: false,
          },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
        ],
        data: Buffer.from(data),
      });

      // Compute budget — ZK verifier требует много CU
      const computeBudgetInstruction = ComputeBudgetProgram.setComputeUnitLimit({
        units: 1_400_000,
      });

      const { blockhash } = await connection.getLatestBlockhash("confirmed");
      const messageV0 = new TransactionMessage({
        payerKey: userPublicKey,
        recentBlockhash: blockhash,
        instructions: [computeBudgetInstruction, withdrawInstruction],
      }).compileToV0Message();

      const transaction = new VersionedTransaction(messageV0);

      // 12. Подписываем напрямую через signTransaction (без симуляции Phantom)
      if (typeof provider.signTransaction !== "function") {
        throw new Error("Wallet does not support signTransaction");
      }

      const signedTransaction = await provider.signTransaction(transaction);
      const signature = await connection.sendRawTransaction(signedTransaction.serialize(), {
        preflightCommitment: "confirmed",
        skipPreflight: true,
      });

      txSignature.value = signature;

      // 13. Помечаем ноту использованной по ЛОКАЛЬНОМУ SHA-256-хешу
      depositsStore.markUsed(localNullifierHashHex);

      const balance = await connection.getBalance(userPublicKey);
      walletStore.setBalance(BigInt(balance));
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Withdraw failed";
      console.error("Withdraw error:", err);
      throw err;
    } finally {
      loading.value = false;
      proofGenerating.value = false;
    }
  }

  function reset() {
    loading.value = false;
    error.value = null;
    txSignature.value = null;
    proofGenerating.value = false;
  }

  return {
    loading,
    error,
    txSignature,
    proofGenerating,
    withdraw,
    reset,
  };
}
