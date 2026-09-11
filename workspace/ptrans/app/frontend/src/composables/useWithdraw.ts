import { ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { useDepositsStore } from "@/stores/deposits";
import { hexToBytes, bytesToHex } from "@/services/crypto";
import { computeNullifierHash } from "@/services/poseidon";
import { withdraw as apiWithdraw } from "@/services/api";
import {
  Connection,
  PublicKey,
  TransactionMessage,
  VersionedTransaction,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import { Buffer } from "buffer";
import { findPoolPda, findPoolVaultPda, findNullifierSetPda } from "@/generated/pdas";
import { WITHDRAW_DISCRIMINATOR } from "@/generated/instructions/withdraw";
import { PTRANS_PROGRAM_ADDRESS } from "@/generated/programs";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";
const VERIFIER_PROGRAM_ID =
  import.meta.env.VITE_VERIFIER_PROGRAM_ID || "AdTqk4n6ifgUkAHA77SBKKKvQxsYyofqUp7LVQVQKac";

export function useWithdraw() {
  const walletStore = useWalletStore();
  const depositsStore = useDepositsStore();

  const loading = ref(false);
  const error = ref<string | null>(null);
  const txSignature = ref<string | null>(null);
  const proofGenerating = ref(false);

  // Копия getProvider() из useWallet.ts — возвращает провайдер кошелька
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

      // Локальный SHA-256 — только для поиска ноты в localStorage
      const nullifierSecretBytes = hexToBytes(nullifierSecretHex);
      const localNullifierHash = await computeNullifierHash(nullifierSecretBytes);
      const localNullifierHashHex = bytesToHex(localNullifierHash);

      const note = depositsStore.getNoteByNullifier(localNullifierHashHex);
      if (!note) {
        throw new Error("Deposit note not found");
      }
      if (note.used) {
        throw new Error("This deposit has already been withdrawn");
      }

      // Запрос к бэкенду за ZK-proof
      const withdrawResponse = await apiWithdraw({
        nullifierSecret: nullifierSecretHex,
        secret: secretHex,
        amount: Number(amountLamports),
        recipient: recipientAddress,
      });

      proofGenerating.value = false;

      // PDA-адреса
      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });
      const [nullifierSetPda] = await findNullifierSetPda({ pool: poolPda });

      // Декодируем proof (base64 → Uint8Array)
      const proofBytes = Uint8Array.from(atob(withdrawResponse.proof), (c) => c.charCodeAt(0));

      // Poseidon-хеши от бэкенда (совпадают со схемой Noir)
      const nullifierHashBytes = hexToBytes(withdrawResponse.nullifierHash);
      const rootBytes = hexToBytes(withdrawResponse.root);

      // Создаем инструкцию вручную через @solana/web3.js
      const connection = new Connection(RPC_URL, "confirmed");
      const userPublicKey = new PublicKey(walletStore.walletAddress);
      const recipientPubkey = new PublicKey(recipientAddress);

      // Кодируем data инструкции (Borsh-совместимо):
      // discriminator (8) + Vec<u8> proof (4 + N) + nullifierHash (32) + root (32) + to (32) + amount (8)
      const proofLen = proofBytes.length;
      const data = new Uint8Array(8 + 4 + proofLen + 32 + 32 + 32 + 8);
      let offset = 0;

      data.set(WITHDRAW_DISCRIMINATOR, offset);
      offset += 8;

      // Vec<u8> в Borsh: u32 длина (LE) + байты
      new DataView(data.buffer).setUint32(offset, proofLen, true);
      offset += 4;
      data.set(proofBytes, offset);
      offset += proofLen;

      data.set(nullifierHashBytes, offset);
      offset += 32;
      data.set(rootBytes, offset);
      offset += 32;
      data.set(recipientPubkey.toBytes(), offset);
      offset += 32;

      new DataView(data.buffer).setBigUint64(offset, amountLamports, true);

      const withdrawInstruction = new TransactionInstruction({
        programId: new PublicKey(PTRANS_PROGRAM_ADDRESS),
        keys: [
          { pubkey: new PublicKey(poolPda), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(nullifierSetPda), isSigner: false, isWritable: true },
          { pubkey: new PublicKey(vaultPda), isSigner: false, isWritable: true },
          { pubkey: recipientPubkey, isSigner: false, isWritable: true },
          { pubkey: new PublicKey(VERIFIER_PROGRAM_ID), isSigner: false, isWritable: false },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        data: Buffer.from(data),
      });

      const { blockhash } = await connection.getLatestBlockhash("confirmed");
      const messageV0 = new TransactionMessage({
        payerKey: userPublicKey,
        recentBlockhash: blockhash,
        instructions: [withdrawInstruction],
      }).compileToV0Message();

      const transaction = new VersionedTransaction(messageV0);

      // Подпись и отправка — единый путь для Phantom и Solflare
      let signature: string;

      if (typeof provider.signAndSendTransaction === "function") {
        const result = await provider.signAndSendTransaction(transaction);
        signature = typeof result === "string" ? result : result.signature;
      } else if (typeof provider.signTransaction === "function") {
        const signedTx = await provider.signTransaction(transaction);
        signature = await connection.sendRawTransaction(signedTx.serialize(), {
          preflightCommitment: "confirmed",
        });
      } else {
        throw new Error("Wallet does not support transaction signing");
      }

      txSignature.value = signature;

      // Помечаем ноту использованной по ЛОКАЛЬНОМУ SHA-256-хешу
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
