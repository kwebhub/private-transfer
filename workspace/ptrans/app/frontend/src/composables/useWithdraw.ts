import { ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { useDepositsStore } from "@/stores/deposits";
import { hexToBytes, bytesToHex } from "@/services/crypto";
import { computeNullifierHash, generateWithdrawalWitness } from "@/services/poseidon";
import { withdraw as apiWithdraw, getCommitments, getProof } from "@/services/api";
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

const BN254_MODULUS =
  21888242871839275222246405745257275088548364400416034343698204186575808495617n;

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

      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });
      const [nullifierSetPda] = await findNullifierSetPda({ pool: poolPda });

      // 1. Получаем commitments из backend
      const commitmentsResponse = await getCommitments(poolPda);
      const sortedCommitments = commitmentsResponse.commitments
        .sort((a, b) => a.leaf_index - b.leaf_index)
        .map((e) => hexToBytes(e.commitment));

      if (sortedCommitments.length === 0) {
        throw new Error("No commitments in database");
      }

      // 2. Находим leaf_index для нашего commitment'а
      const noteCommitmentHex = note.commitment.replace(/^0x/, "").toLowerCase();
      const leafIndex = sortedCommitments.findIndex(
        (c) => bytesToHex(c).toLowerCase() === noteCommitmentHex,
      );

      if (leafIndex === -1) {
        throw new Error("Your commitment not found in pool");
      }

      // 3. Получаем Merkle proof из backend (проксируется в merkle-сервис)
      const proofResponse = await getProof(poolPda, leafIndex);
      const merkleProof = proofResponse.proof.map((h) => hexToBytes(h));
      const isEven = proofResponse.is_even;
      const latestRoot = hexToBytes(proofResponse.root);

      // 4. Вычисляем nullifier_hash через Noir
      const nullifierHashBytes = await computeNullifierHash(nullifierSecretBytes);

      // 5. Recipient
      const recipientPubkey = new PublicKey(recipientAddress);
      const recipientRealBytes = recipientPubkey.toBytes();
      const recipientReducedBytes = reduceToField(recipientRealBytes);

      // 6. Генерируем witness
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

      const witnessB64 = Buffer.from(witness).toString("base64");
      const withdrawResponse = await apiWithdraw({ witness: witnessB64 });

      const proofBytes = Buffer.from(withdrawResponse.proof, "base64");
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

      const connection = new Connection(RPC_URL, {
        commitment: "confirmed",
        confirmTransactionInitialTimeout: 30_000,
      });
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

      const computeBudgetInstruction = ComputeBudgetProgram.setComputeUnitLimit({
        units: 1_400_000,
      });

      const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
      const messageV0 = new TransactionMessage({
        payerKey: userPublicKey,
        recentBlockhash: blockhash,
        instructions: [computeBudgetInstruction, withdrawInstruction],
      }).compileToV0Message();

      const transaction = new VersionedTransaction(messageV0);

      if (typeof provider.signTransaction !== "function") {
        throw new Error("Wallet does not support signTransaction");
      }

      const signedTransaction = await provider.signTransaction(transaction);
      const signature = await connection.sendRawTransaction(signedTransaction.serialize(), {
        preflightCommitment: "confirmed",
        skipPreflight: true,
        maxRetries: 5,
      });

      const confirmation = await connection.confirmTransaction(
        { signature, blockhash, lastValidBlockHeight },
        "confirmed",
      );

      if (confirmation.value.err) {
        throw new Error(`Transaction failed on-chain: ${JSON.stringify(confirmation.value.err)}`);
      }

      txSignature.value = signature;

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
