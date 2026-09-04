import { ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { useWallet } from "@/composables/useWallet";
import { useDepositsStore } from "@/stores/deposits";
import { hexToBytes, computeNullifierHash, bytesToHex } from "@/services/crypto";
import { withdraw as apiWithdraw } from "@/services/api";
import {
  createSolanaRpc,
  address as solanaAddress,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  pipe,
  getSignatureFromTransaction
} from "@solana/kit";
import { getWithdrawInstructionAsync } from "@/generated/instructions";
import { findPoolPda, findPoolVaultPda, findNullifierSetPda } from "@/generated/pdas";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";
const VERIFIER_PROGRAM_ID =
  import.meta.env.VITE_VERIFIER_PROGRAM_ID || "AdTqk4n6ifgUkAHA77SBKKKvQxsYyofqUp7LVQVQKac";

export function useWithdraw() {
  const walletStore = useWalletStore();
  const depositsStore = useDepositsStore();

  // Подключаем useWallet для получения живого интерфейса кошелька Wallet Standard
  const { getActiveWalletInterface } = useWallet();

  const loading = ref(false);
  const error = ref<string | null>(null);
  const txSignature = ref<string | null>(null);
  const proofGenerating = ref(false);

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
      // Получаем инстанс кошелька, зарегистрированный в браузере
      const activeWallet = getActiveWalletInterface();

      // 1. Проверяем подключение кошелька
      if (!walletStore.isConnected || !walletStore.walletAddress || !activeWallet) {
        throw new Error("Wallet not connected");
      }

      const amountLamports = BigInt(Math.floor(amountSol * 1_000_000_000));

      // 2. Вычисляем nullifierHash для валидации
      const nullifierSecretBytes = hexToBytes(nullifierSecretHex);
      const nullifierHash = computeNullifierHash(nullifierSecretBytes);
      const nullifierHashHex = bytesToHex(nullifierHash);

      // 3. Проверяем, что записка существует и не использована
      const note = depositsStore.getNoteByNullifier(nullifierHashHex);
      if (!note) {
        throw new Error("Deposit note not found");
      }
      if (note.used) {
        throw new Error("This deposit has already been withdrawn");
      }

      // 4. Запрос на бэкенд для генерации ZK-доказательства (Noir)
      const withdrawResponse = await apiWithdraw({
        nullifierSecret: nullifierSecretHex,
        secret: secretHex,
        amount: Number(amountLamports),
        recipient: recipientAddress,
      });

      proofGenerating.value = false;

      // 5. Инициализируем RPC-клиент v2
      const rpc = createSolanaRpc(RPC_URL);

      // 6. Получаем PDA-адреса контракта
      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });
      const [nullifierSetPda] = await findNullifierSetPda({ pool: poolPda });

      // 7. Декодируем proof и публичные инпуты из hex/base64 в Uint8Array
      const proofBytes = Uint8Array.from(atob(withdrawResponse.proof), (c) => c.charCodeAt(0));
      const nullifierHashBytes = hexToBytes(withdrawResponse.nullifierHash);
      const rootBytes = hexToBytes(withdrawResponse.root);

      // Типизируем адреса под формат Address для SDK v2
      const recipientPubkey = solanaAddress(recipientAddress);
      const userAddress = solanaAddress(walletStore.walletAddress);
      const verifierProgramAddress = solanaAddress(VERIFIER_PROGRAM_ID);

      // 8. Генерируем асинхронную инструкцию вывода через Codama-клиент
      const withdrawInstruction = await getWithdrawInstructionAsync({
        pool: poolPda,
        nullifierSet: nullifierSetPda,
        poolVault: vaultPda,
        recipient: recipientPubkey,
        verifierProgram: verifierProgramAddress,
        proof: proofBytes,
        nullifierHash: nullifierHashBytes,
        root: rootBytes,
        amount: amountLamports
      });

      // 9. Получаем свежий блокхеш
      const { value: latestBlockhash } = await rpc.getLatestBlockhash({ commitment: 'confirmed' }).send();

      // 10. Собираем транзакцию с помощью pipe
      const transactionMessage = pipe(
        createTransactionMessage({ version: 0 }),
        (m) => setTransactionMessageFeePayer(userAddress, m),
        (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
        (m) => appendTransactionMessageInstruction(withdrawInstruction, m)
      );

      // 11. Подписываем транзакцию через Wallet Standard API
      const signProperty = activeWallet.features["solana:signTransaction"];
      if (!signProperty) {
        throw new Error("Connected wallet does not support signing transactions.");
      }

      const [signedTransaction] = await signProperty.signTransaction([transactionMessage]);
      const signature = getSignatureFromTransaction(signedTransaction);

      // 12. Отправляем транзакцию в сеть
      await rpc.sendTransaction(signedTransaction, { encoding: 'base64', preflightCommitment: 'confirmed' }).send();
      txSignature.value = signature;

      // 13. Отмечаем записку как использованную
      depositsStore.markUsed(withdrawResponse.nullifierHash);

      // 14. Обновляем баланс кошелька
      const balanceResponse = await rpc.getBalance(userAddress).send();
      walletStore.setBalance(balanceResponse.value);
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
