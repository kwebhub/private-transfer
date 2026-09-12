import { ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { useDepositsStore } from "@/stores/deposits";
import { generateSecrets, hexToBytes, bytesToHex } from "@/services/crypto";
import { computeCommitment } from "@/services/poseidon";
import {
  Connection,
  PublicKey,
  TransactionMessage,
  VersionedTransaction,
  SystemProgram,
  TransactionInstruction,
} from "@solana/web3.js";
import { Buffer } from "buffer";
import { findPoolPda, findPoolVaultPda } from "@/generated/pdas";
import { DEPOSIT_DISCRIMINATOR } from "@/generated/instructions/deposit";
import { PTRANS_PROGRAM_ADDRESS } from "@/generated/programs";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";
const MERKLE_URL = import.meta.env.VITE_MERKLE_URL || "http://localhost:4003";
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
    if ((window as any).phantom?.solana) return (window as any).phantom.solana;
    if ((window as any).solflare) return (window as any).solflare;
    if ((window as any).solana) return (window as any).solana;
    return null;
  }

  // ================================================================
  // Получение всех commitments из событий DepositEvent через RPC
  // ================================================================
  async function fetchAllCommitmentsFromChain(poolAddress: string): Promise<Uint8Array[]> {
    const rpc = (body: unknown) =>
      fetch(RPC_URL, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      }).then((r) => r.json());

    const sigsResp = await rpc({
      jsonrpc: "2.0",
      id: 1,
      method: "getSignaturesForAddress",
      params: [poolAddress, { limit: 1000 }],
    });

    const sigs = sigsResp.result || [];

    const entries: { leafIndex: number; commitment: Uint8Array }[] = [];

    for (const sig of sigs) {
      const txResp = await rpc({
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

        entries.push({ leafIndex, commitment });
      }
    }

    entries.sort((a, b) => a.leafIndex - b.leafIndex);
    return entries.map((e) => e.commitment);
  }

  // ================================================================
  // Запрос нового корня у merkle-сервиса (Noir Poseidon2)
  // ================================================================
  async function fetchNewRootFromMerkle(
    newCommitment: Uint8Array,
    existingCommitments: Uint8Array[],
  ): Promise<Uint8Array> {
    const allCommitments = [...existingCommitments, newCommitment];
    const commitmentsHex = allCommitments.map((c) => bytesToHex(c));

    const resp = await fetch(`${MERKLE_URL}/root`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ commitments: commitmentsHex }),
    });

    if (!resp.ok) {
      const err = await resp.text();
      throw new Error(`Merkle service error: ${resp.status} ${err}`);
    }

    const { root } = await resp.json();
    return hexToBytes(root);
  }

  // ================================================================
  // Депозит
  // ================================================================
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

      // 1. PDA-адреса
      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });

      // 2. Получаем все существующие commitments из событий
      const existingCommitments = await fetchAllCommitmentsFromChain(poolPda);

      // 3. Получаем новый корень из merkle-сервиса
      const targetNewRoot = await fetchNewRootFromMerkle(commitment, existingCommitments);

      // 4. Создаём транзакцию через @solana/web3.js
      const connection = new Connection(RPC_URL, "confirmed");
      const userPublicKey = new PublicKey(walletStore.walletAddress);

      // data: discriminator(8) + commitment(32) + new_root(32) + amount(8)
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

      // 5. Подписываем и отправляем (единый путь для Phantom и Solflare)
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

      // 6. Сохраняем ноту
      const note = depositsStore.addNote(
        nullifierSecret,
        secret,
        Number(amountLamports),
        commitment,
        nullifierHash,
      );
      depositNote.value = note;

      // 7. Обновляем баланс
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
