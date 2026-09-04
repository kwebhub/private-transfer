import { ref, onMounted } from "vue";
import { createSolanaRpc, address as solanaAddress } from "@solana/kit";
import { findPoolPda, findPoolVaultPda } from "@/generated/pdas";
import { fetchPoolAcc } from "@/generated/accounts"; // Сама Codama генерирует эту функцию fetch
import { bytesToHex } from "@/services/crypto";
import { getPoolInfo } from "@/services/api";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://solana.com";

export interface PoolInfo {
  address: string;
  vaultAddress: string;
  vaultBalance: number;
  vaultBalanceSol: number;
  nextLeafIndex: number;
  totalDeposits: number;
  currentRoot: string;
  nullifiersCount: number;
}

export function usePool() {
  const loading = ref(false);
  const error = ref<string | null>(null);
  const poolInfo = ref<PoolInfo | null>(null);

  async function fetchPoolInfo(): Promise<void> {
    loading.value = true;
    error.value = null;

    try {
      // Инициализируем RPC-клиент нового поколения
      const rpc = createSolanaRpc(RPC_URL);

      // 1. Находим PDA-адреса
      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });

      // 2. Получаем баланс хранилища (в v2 вызовы делаются через .send())
      const vaultBalanceResponse = await rpc.getBalance(solanaAddress(vaultPda)).send();
      const vaultBalance = Number(vaultBalanceResponse.value);

      // 3. Получаем и автоматически декодируем аккаунт пула
      let nextLeafIndex = 0;
      let totalDeposits = 0;
      let currentRoot = "0x" + "0".repeat(64);

      try {
        // fetchPoolAcc сама идет по RPC, берет данные и парсит их структуру по IDL
        const poolAccount = await fetchPoolAcc(rpc, poolPda);

        nextLeafIndex = Number(poolAccount.data.nextLeafIndex);
        totalDeposits = Number(poolAccount.data.totalDeposits);

        // Получаем текущий хэш корня из массива истории roots
        const rootIndex = Number(poolAccount.data.currentRootIndex);
        if (poolAccount.data.roots && poolAccount.data.roots[rootIndex]) {
          currentRoot = "0x" + bytesToHex(poolAccount.data.roots[rootIndex]);
        }
      } catch (err) {
        // Аккаунт пула еще не инициализирован в блокчейне
        console.warn("Pool account not found or not initialized yet:", err);
      }

      // 4. Получаем информацию со вспомогательного ZK-бэкенда
      let nullifiersCount = 0;
      try {
        const backendInfo = await getPoolInfo();
        nullifiersCount = backendInfo.nullifiersCount;
      } catch (err) {
        console.warn("Failed to get pool info from backend:", err);
      }

      // Сохраняем агрегированное состояние для интерфейса Vue
      poolInfo.value = {
        address: poolPda.toString(),
        vaultAddress: vaultPda.toString(),
        vaultBalance,
        vaultBalanceSol: vaultBalance / 1_000_000_000,
        nextLeafIndex,
        totalDeposits,
        currentRoot,
        nullifiersCount,
      };
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Failed to fetch pool info";
      console.error("Fetch pool info error:", err);
    } finally {
      loading.value = false;
    }
  }

  onMounted(() => {
    fetchPoolInfo();
  });

  return {
    loading,
    error,
    poolInfo,
    fetchPoolInfo,
  };
}
