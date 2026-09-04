import { ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { useWallet } from "@/composables/useWallet";
import { useDepositsStore } from "@/stores/deposits";
import { generateSecrets, computeCommitment, computeNullifierHash } from "@/services/crypto";
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
import { getDepositInstructionAsync } from "@/generated/instructions";
import { findPoolPda, findPoolVaultPda } from "@/generated/pdas";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";

export function useDeposit() {
  const walletStore = useWalletStore();
  const depositsStore = useDepositsStore();
  const { getActiveWalletInterface } = useWallet();

  const loading = ref(false);
  const error = ref<string | null>(null);
  const txSignature = ref<string | null>(null);
  const depositNote = ref<any>(null);

  const MIN_DEPOSIT = 0.001; // SOL

  async function deposit(amountSol: number): Promise<void> {
    loading.value = true;
    error.value = null;
    txSignature.value = null;
    depositNote.value = null;

    try {
      if (!walletStore.isConnected || !walletStore.walletAddress) {
        throw new Error("Wallet not connected");
      }

      if (amountSol < MIN_DEPOSIT) {
        throw new Error(`Minimum deposit is ${MIN_DEPOSIT} SOL`);
      }

      const amountLamports = BigInt(Math.floor(amountSol * 1_000_000_000));
      const { nullifierSecret, secret } = generateSecrets();
      const commitment = computeCommitment(nullifierSecret, secret, amountLamports);
      const nullifierHash = computeNullifierHash(nullifierSecret);
      const newRoot = new Uint8Array(32);

      const rpc = createSolanaRpc(RPC_URL);

      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });
      const userAddress = solanaAddress(walletStore.walletAddress);

      const depositInstruction = await getDepositInstructionAsync({
        pool: poolPda,
        vault: vaultPda,
        user: userAddress,
        commitment: commitment,
        newRoot: newRoot,
        amount: amountLamports
      });

      const { value: latestBlockhash } = await rpc.getLatestBlockhash({ commitment: 'confirmed' }).send();

      const transactionMessage = pipe(
        createTransactionMessage({ version: 0 }),
        (m) => setTransactionMessageFeePayer(userAddress, m),
        (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
        (m) => appendTransactionMessageInstruction(depositInstruction, m)
      );

      let signature: string;
      const activeWallet = getActiveWalletInterface();

      if (activeWallet && activeWallet.features["solana:signTransaction"]) {
        // Подписание по Wallet Standard
        const signProperty = activeWallet.features["solana:signTransaction"];
        const [signedTransaction] = await signProperty.signTransaction([transactionMessage]);
        signature = getSignatureFromTransaction(signedTransaction);
        await rpc.sendTransaction(signedTransaction, { encoding: 'base64', preflightCommitment: 'confirmed' }).send();
      } else {
        // Резервный метод подписи через глобальный window.solana
        const provider = (window as any).phantom?.solana || (window as any).solana || (window as any).solflare;
        if (!provider) throw new Error("Wallet provider missing during signing.");

        const signedTx = await provider.signTransaction(transactionMessage);
        signature = getSignatureFromTransaction(signedTx);
        await rpc.sendTransaction(signedTx, { encoding: 'base64', preflightCommitment: 'confirmed' }).send();
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

      const balanceResponse = await rpc.getBalance(userAddress).send();
      walletStore.setBalance(balanceResponse.value);

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
