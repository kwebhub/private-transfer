import { ref, onMounted } from "vue";
import {
  createSolanaRpc,
  address as solanaAddress,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  pipe,
  compileTransactionMessage,
  getSignatureFromTransaction,
} from "@solana/kit";
import { findPoolPda, findPoolVaultPda } from "@/generated/pdas";
import { fetchPoolAcc } from "@/generated/accounts";
import { getPoolInstructionAsync } from "@/generated/instructions/pool";
import { bytesToHex } from "@/services/crypto";
import { useWalletStore } from "@/stores/wallet";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";

export interface PoolInfo {
  address: string;
  vaultAddress: string;
  vaultBalance: number;
  vaultBalanceSol: number;
  nextLeafIndex: number;
  totalDeposits: number;
  currentRoot: string;
  nullifiersCount: number;
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
    return (window as any).phantom?.solana || (window as any).solana || (window as any).solflare;
  }

  async function fetchPoolInfo(): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      const rpc = createSolanaRpc(RPC_URL);
      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });

      // Получаем баланс хранилища
      const vaultBalanceResponse = await rpc.getBalance(solanaAddress(vaultPda)).send();
      const vaultBalance = Number(vaultBalanceResponse.value);

      // Пытаемся загрузить аккаунт пула
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
          currentRoot = "0x" + bytesToHex(poolAccount.data.roots[rootIndex]);
        }
      } catch (err) {
        // Аккаунт пула не инициализирован
        console.warn("Pool account not found or not initialized yet", err);
        isInitialized = false;
      }

      // Получаем количество nullifier'ов с бэкенда (опционально)
      let nullifiersCount = 0;
      try {
        // Если есть API для получения информации о пуле
        // const backendInfo = await getPoolInfo();
        // nullifiersCount = backendInfo.nullifiersCount;
      } catch (err) {
        console.warn("Failed to get pool info from backend:", err);
      }

      poolInfo.value = {
        address: poolPda.toString(),
        vaultAddress: vaultPda.toString(),
        vaultBalance,
        vaultBalanceSol: vaultBalance / 1_000_000_000,
        nextLeafIndex,
        totalDeposits,
        currentRoot,
        nullifiersCount,
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

      const rpc = createSolanaRpc(RPC_URL);
      const authorityAddress = solanaAddress(walletStore.walletAddress);

      // Создаем signer для провайдера
      const authoritySigner = {
        address: authorityAddress,
        signTransactions: async (transactions: any[]) => {
          return await Promise.all(
            transactions.map(async (tx) => {
              return await provider.signTransaction(tx);
            }),
          );
        },
      };

      // Создаем инструкцию инициализации пула
      const poolInstruction = await getPoolInstructionAsync({
        authority: authoritySigner,
      });

      // Получаем последний блокхэш
      const { value: latestBlockhash } = await rpc
        .getLatestBlockhash({ commitment: "confirmed" })
        .send();

      // Создаем транзакцию
      let transactionMessage = pipe(
        createTransactionMessage({ version: 0 }),
        (m) => setTransactionMessageFeePayer(authorityAddress, m),
        (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
        (m) => appendTransactionMessageInstruction(poolInstruction, m),
      );

      // Компилируем и подписываем
      const compiledMessage = compileTransactionMessage(transactionMessage);
      const signedTransaction = await provider.signTransaction({
        message: compiledMessage,
        signatures: [new Uint8Array(64)],
      });

      // Получаем сигнатуру
      const signature = getSignatureFromTransaction(signedTransaction);

      // Отправляем транзакцию
      await rpc
        .sendTransaction(signedTransaction, {
          encoding: "base64",
          preflightCommitment: "confirmed",
        })
        .send();

      console.log("✅ Pool initialized successfully!");
      console.log("📝 Signature:", signature);

      // Обновляем информацию о пуле
      await fetchPoolInfo();
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Failed to initialize pool";
      console.error("Initialize pool error:", err);
      throw err;
    } finally {
      initializing.value = false;
    }
  }

  // Автоматически загружаем информацию о пуле при монтировании
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
