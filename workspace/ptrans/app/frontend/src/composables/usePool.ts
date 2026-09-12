import { ref, onMounted } from "vue";
import { createSolanaRpc, address as solanaAddress } from "@solana/kit";
import { Buffer } from "buffer";
import { findPoolPda, findPoolVaultPda, findNullifierSetPda } from "@/generated/pdas";
import { fetchPoolAcc } from "@/generated/accounts";
import { getPoolInstructionAsync } from "@/generated/instructions/pool";
import { bytesToHex } from "@/services/crypto";
import { useWalletStore } from "@/stores/wallet";
import { Connection, PublicKey, TransactionMessage, VersionedTransaction } from "@solana/web3.js";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";

export interface PoolInfo {
  address: string;
  vaultAddress: string;
  nullifierSetAddress: string;
  vaultBalance: number;
  vaultBalanceSol: number;
  nextLeafIndex: number;
  totalDeposits: number;
  currentRoot: string;
  isInitialized: boolean;
}

export function usePool() {
  const walletStore = useWalletStore();
  const loading = ref(false);
  const initializing = ref(false);
  const error = ref<string | null>(null);
  const poolInfo = ref<PoolInfo | null>(null);

  function getProvider() {
    if (typeof window === "undefined") return null;
    if ((window as any).phantom?.solana) return (window as any).phantom.solana;
    if ((window as any).solflare) return (window as any).solflare;
    if ((window as any).solana) return (window as any).solana;
    return null;
  }

  async function fetchPoolInfo(): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      const rpc = createSolanaRpc(RPC_URL);
      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });
      const [nullifierSetPda] = await findNullifierSetPda({ pool: poolPda });

      const vaultBalanceResponse = await rpc.getBalance(solanaAddress(vaultPda)).send();
      const vaultBalance = Number(vaultBalanceResponse.value);

      let nextLeafIndex = 0;
      let totalDeposits = 0;
      let currentRoot = "0x" + "0".repeat(64);
      let isInitialized = false;

      try {
        const poolAccount = await fetchPoolAcc(rpc, poolPda);
        isInitialized = true;
        nextLeafIndex = Number(poolAccount.data.nextLeafIndex);
        totalDeposits = Number(poolAccount.data.totalDeposits);

        const rootIndex = Number(poolAccount.data.currentRootIndex);
        if (poolAccount.data.roots && poolAccount.data.roots[rootIndex]) {
          currentRoot = "0x" + bytesToHex(poolAccount.data.roots[rootIndex] as Uint8Array);
        }
      } catch (err) {
        console.warn("Pool account not found or not initialized yet:", err);
        isInitialized = false;
      }

      poolInfo.value = {
        address: poolPda.toString(),
        vaultAddress: vaultPda.toString(),
        nullifierSetAddress: nullifierSetPda.toString(),
        vaultBalance,
        vaultBalanceSol: vaultBalance / 1_000_000_000,
        nextLeafIndex,
        totalDeposits,
        currentRoot,
        isInitialized,
      };
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Failed to fetch pool info";
      console.error("Fetch pool info error:", err);
    } finally {
      loading.value = false;
    }
  }

  async function initializePool(): Promise<void> {
    initializing.value = true;
    error.value = null;

    try {
      const provider = getProvider();
      if (!walletStore.isConnected || !walletStore.walletAddress || !provider) {
        throw new Error("Wallet not connected");
      }

      const userPublicKey = new PublicKey(walletStore.walletAddress);
      const connection = new Connection(RPC_URL, "confirmed");

      // 1. Создаём instruction через Codama
      const authoritySigner = {
        address: solanaAddress(walletStore.walletAddress),
        signTransactions: async (txs: any[]) =>
          Promise.all(txs.map((tx: any) => provider.signTransaction(tx))),
      };

      const poolInstruction = await getPoolInstructionAsync({
        authority: authoritySigner as any,
      });

      // 2. Преобразуем Codama instruction в web3.js instruction
      const keys = poolInstruction.accounts.map((acc: any) => ({
        pubkey: new PublicKey(acc.address),
        isSigner: acc.role === 2 || acc.role === 3,
        isWritable: acc.role === 1 || acc.role === 3,
      }));

      const { TransactionInstruction } = await import("@solana/web3.js");
      const ix = new TransactionInstruction({
        programId: new PublicKey(poolInstruction.programAddress),
        keys,
        data: Buffer.from(poolInstruction.data),
      });

      // 3. Собираем транзакцию
      const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash("confirmed");
      const messageV0 = new TransactionMessage({
        payerKey: userPublicKey,
        recentBlockhash: blockhash,
        instructions: [ix],
      }).compileToV0Message();

      const transaction = new VersionedTransaction(messageV0);

      // 4. Подписываем через signTransaction (без симуляции Phantom)
      if (typeof provider.signTransaction !== "function") {
        throw new Error("Wallet does not support signTransaction");
      }

      const signedTx = await provider.signTransaction(transaction);
      const signature = await connection.sendRawTransaction(signedTx.serialize(), {
        preflightCommitment: "confirmed",
        skipPreflight: true,
        maxRetries: 5,
      });

      const confirmation = await connection.confirmTransaction(
        { signature, blockhash, lastValidBlockHeight },
        "confirmed",
      );

      if (confirmation.value.err) {
        throw new Error(`Transaction failed: ${JSON.stringify(confirmation.value.err)}`);
      }

      console.log("✅ Pool initialized:", signature);

      await fetchPoolInfo();
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Failed to initialize pool";
      console.error("Initialize pool error:", err);
      throw err;
    } finally {
      initializing.value = false;
    }
  }

  onMounted(() => {
    fetchPoolInfo();
  });

  return {
    loading,
    initializing,
    error,
    poolInfo,
    fetchPoolInfo,
    initializePool,
  };
}
