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
      const activeWallet = getActiveWalletInterface();
      if (!walletStore.isConnected || !walletStore.walletAddress || !activeWallet) {
        throw new Error("Wallet not connected");
      }

      const amountLamports = BigInt(Math.floor(amountSol * 1_000_000_000));

      const nullifierSecretBytes = hexToBytes(nullifierSecretHex);
      const nullifierHash = computeNullifierHash(nullifierSecretBytes);
      const nullifierHashHex = bytesToHex(nullifierHash);

      const note = depositsStore.getNoteByNullifier(nullifierHashHex);
      if (!note) {
        throw new Error("Deposit note not found");
      }
      if (note.used) {
        throw new Error("This deposit has already been withdrawn");
      }

      const withdrawResponse = await apiWithdraw({
        nullifierSecret: nullifierSecretHex,
        secret: secretHex,
        amount: Number(amountLamports),
        recipient: recipientAddress,
      });

      proofGenerating.value = false;

      const rpc = createSolanaRpc(RPC_URL);

      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });
      const [nullifierSetPda] = await findNullifierSetPda({ pool: poolPda });

      const proofBytes = Uint8Array.from(atob(withdrawResponse.proof), (c) => c.charCodeAt(0));
      const nullifierHashBytes = hexToBytes(withdrawResponse.nullifierHash);
      const rootBytes = hexToBytes(withdrawResponse.root);

      const recipientPubkey = solanaAddress(recipientAddress);
      const userAddress = solanaAddress(walletStore.walletAddress);
      const verifierProgramAddress = solanaAddress(VERIFIER_PROGRAM_ID);

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

      const { value: latestBlockhash } = await rpc.getLatestBlockhash({ commitment: 'confirmed' }).send();

      const transactionMessage = pipe(
        createTransactionMessage({ version: 0 }),
        (m) => setTransactionMessageFeePayer(userAddress, m),
        (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
        (m) => appendTransactionMessageInstruction(withdrawInstruction, m)
      );

      const signProperty = activeWallet.features["solana:signTransaction"];
      if (!signProperty) {
        throw new Error("Connected wallet does not support signing transactions via Wallet Standard.");
      }

      const [signedTransaction] = await signProperty.signTransaction([transactionMessage]);
      const signature = getSignatureFromTransaction(signedTransaction);

      await rpc.sendTransaction(signedTransaction, { encoding: 'base64', preflightCommitment: 'confirmed' }).send();
      txSignature.value = signature;

      depositsStore.markUsed(withdrawResponse.nullifierHash);

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
