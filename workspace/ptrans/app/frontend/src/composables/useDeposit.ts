import { ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { useDepositsStore } from "@/stores/deposits";
import { generateSecrets } from "@/services/crypto";
import { computeCommitment, poseidon2Hash } from "@/services/poseidon";
import { Buffer } from "buffer";
import {
  Connection,
  PublicKey,
  TransactionMessage,
  VersionedTransaction,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import { createSolanaRpc } from "@solana/kit";
import { findPoolPda, findPoolVaultPda } from "@/generated/pdas";
import { fetchPoolAcc } from "@/generated/accounts";
import { DEPOSIT_DISCRIMINATOR } from "@/generated/instructions/deposit";
import { PTRANS_PROGRAM_ADDRESS } from "@/generated/programs";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";
const MERKLE_TREE_DEPTH = Number(import.meta.env.VITE_MERKLE_TREE_DEPTH);
const MIN_DEPOSIT = Number(import.meta.env.VITE_MIN_DEPOSIT);

export function useDeposit() {
  const walletStore = useWalletStore();
  const depositsStore = useDepositsStore();

  const loading = ref(false);
  const error = ref<string | null>(null);
  const txSignature = ref<string | null>(null);
  const depositNote = ref<any>(null);

  function getProvider() {
    if (typeof window === "undefined") return null;

    const solflare = (window as any).solflare;
    if (solflare && solflare.isSolflare) {
      return solflare;
    }

    const phantom = (window as any).phantom?.solana;
    if (phantom && phantom.isPhantom) {
      return phantom;
    }

    return (window as any).solana;
  }

  async function calculateNextMerkleRoot(
    nextLeafIndex: number,
    newLeafBytes: Uint8Array,
  ): Promise<Uint8Array> {
    let currentLevelHash = newLeafBytes;
    let index = nextLeafIndex;

    const emptySibling = new Uint8Array(32);

    for (let i = 0; i < MERKLE_TREE_DEPTH; i++) {
      let left: Uint8Array;
      let right: Uint8Array;

      if (index % 2 === 0) {
        left = currentLevelHash;
        right = emptySibling;
      } else {
        left = emptySibling;
        right = currentLevelHash;
      }

      currentLevelHash = await poseidon2Hash(left, right);
      index = Math.floor(index / 2);
    }

    return currentLevelHash;
  }

  async function deposit(amountSol: number): Promise<void> {
    loading.value = true;
    error.value = null;
    txSignature.value = null;
    depositNote.value = null;

    try {
      const provider = getProvider();
      if (!walletStore.isConnected || !walletStore.walletAddress || !provider) {
        throw new Error("Wallet not connected or provider missing");
      }

      if (amountSol < MIN_DEPOSIT) {
        throw new Error(`Minimum deposit is ${MIN_DEPOSIT} SOL`);
      }

      const amountLamports = BigInt(Math.floor(amountSol * 1_000_000_000));
      const { nullifierSecret, secret } = generateSecrets();
      const { commitment, nullifierHash } = await computeCommitment(
        nullifierSecret,
        secret,
        amountLamports,
      );
      const rpc = createSolanaRpc(RPC_URL);
      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });

      const poolAccount = await fetchPoolAcc(rpc, poolPda);
      const nextLeafIndex = Number(poolAccount.data.nextLeafIndex);
      const targetNewRoot = await calculateNextMerkleRoot(nextLeafIndex, commitment);

      const connection = new Connection(RPC_URL, "confirmed");
      const userPublicKey = new PublicKey(walletStore.walletAddress);

      // Формируем data инструкции БЕЗ Buffer — используем Uint8Array
      const data = new Uint8Array(8 + 32 + 32 + 8);
      data.set(DEPOSIT_DISCRIMINATOR, 0);
      data.set(commitment, 8);
      data.set(targetNewRoot, 40);
      new DataView(data.buffer).setBigUint64(72, amountLamports, true);

      const depositInstruction = new TransactionInstruction({
        programId: new PublicKey(PTRANS_PROGRAM_ADDRESS),
        keys: [
          { pubkey: new PublicKey(poolPda), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(vaultPda), isSigner: false, isWritable: true },
          { pubkey: userPublicKey, isSigner: true, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: Buffer.from(data),
      });

      const { blockhash } = await connection.getLatestBlockhash("confirmed");
      const messageV0 = new TransactionMessage({
        payerKey: userPublicKey,
        recentBlockhash: blockhash,
        instructions: [depositInstruction],
      }).compileToV0Message();

      const transaction = new VersionedTransaction(messageV0);

      let signature: string;

      if (typeof provider.signAndSendTransaction === "function") {
        const result = await provider.signAndSendTransaction(transaction);
        signature = typeof result === "string" ? result : result.signature;
      } else if (typeof provider.signTransaction === "function") {
        const signedTransaction = await provider.signTransaction(transaction);
        signature = await connection.sendRawTransaction(signedTransaction.serialize(), {
          preflightCommitment: "confirmed",
        });
      } else {
        throw new Error("Wallet does not support transaction signing");
      }

      txSignature.value = signature;

      const note = depositsStore.addNote(
        nullifierSecret,
        secret,
        Number(amountLamports),
        commitment,
        nullifierHash,
      );
      depositNote.value = note;

      const balance = await connection.getBalance(userPublicKey);
      walletStore.setBalance(BigInt(balance));
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Deposit failed";
      console.error("Deposit error:", err);
      throw err;
    } finally {
      loading.value = false;
    }
  }

  function reset() {
    loading.value = false;
    error.value = null;
    txSignature.value = null;
    depositNote.value = null;
  }

  return {
    loading,
    error,
    txSignature,
    depositNote,
    deposit,
    reset,
    MIN_DEPOSIT,
  };
}
