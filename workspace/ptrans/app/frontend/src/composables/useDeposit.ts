import { ref } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { useDepositsStore } from "@/stores/deposits";
import { generateSecrets, computeCommitment, computeNullifierHash, hash } from "@/services/crypto";
import {
  createSolanaRpc,
  address as solanaAddress,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  pipe,
  compileTransactionMessage,
  getSignatureFromTransaction
} from "@solana/kit";
import { getDepositInstructionAsync } from "@/generated/instructions";
import { findPoolPda, findPoolVaultPda } from "@/generated/pdas";
import { fetchPoolAcc } from "@/generated/accounts";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";
const MERKLE_TREE_DEPTH = 20;

export function useDeposit() {
  const walletStore = useWalletStore();
  const depositsStore = useDepositsStore();

  const loading = ref(false);
  const error = ref<string | null>(null);
  const txSignature = ref<string | null>(null);
  const depositNote = ref<any>(null);

  const MIN_DEPOSIT = 0.001;

  function getProvider() {
    if (typeof window === "undefined") return null;
    return (window as any).phantom?.solana || (window as any).solana || (window as any).solflare;
  }

  function calculateNextMerkleRoot(nextLeafIndex: number, newLeafBytes: Uint8Array): Uint8Array {
    let currentLevelHash = newLeafBytes;
    let index = nextLeafIndex;

    for (let i = 0; i < MERKLE_TREE_DEPTH; i++) {
      const emptyLevelSibling = new Uint8Array(32);
      const pairBuffer = new Uint8Array(64);

      if (index % 2 === 0) {
        pairBuffer.set(currentLevelHash, 0);
        pairBuffer.set(emptyLevelSibling, 32);
      } else {
        pairBuffer.set(emptyLevelSibling, 0);
        pairBuffer.set(currentLevelHash, 32);
      }

      currentLevelHash = hash(pairBuffer);
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
      const commitment = computeCommitment(nullifierSecret, secret, amountLamports);
      const nullifierHash = computeNullifierHash(nullifierSecret);

      const rpc = createSolanaRpc(RPC_URL);

      const [poolPda] = await findPoolPda();
      const [vaultPda] = await findPoolVaultPda({ pool: poolPda });
      const typedUserAddress = solanaAddress(walletStore.walletAddress);

      const poolAccount = await fetchPoolAcc(rpc, poolPda);
      const nextLeafIndex = Number(poolAccount.data.nextLeafIndex);

      const targetNewRoot = calculateNextMerkleRoot(nextLeafIndex, commitment);

      const depositorSigner = {
        address: typedUserAddress,
        signTransactions: async (transactions: any[]) => {
          return await Promise.all(
            transactions.map(async (tx) => {
              return await provider.signTransaction(tx);
            })
          );
        }
      };

      const correctSystemProgramAddress = solanaAddress("11111111111111111111111111111111");

      const depositInstruction = await getDepositInstructionAsync({
        pool: poolPda,
        poolVault: vaultPda,
        depositor: depositorSigner,
        systemProgram: correctSystemProgramAddress,
        commitment: commitment,
        newRoot: targetNewRoot,
        amount: amountLamports
      });

      const { value: latestBlockhash } = await rpc.getLatestBlockhash({ commitment: 'confirmed' }).send();

      const transactionMessage = pipe(
        createTransactionMessage({ version: 0 }),
        (m) => setTransactionMessageFeePayer(typedUserAddress, m),
        (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
        (m) => appendTransactionMessageInstruction(depositInstruction, m)
      );

      const compiledMessage = compileTransactionMessage(transactionMessage);

      const legacyTxMock = {
        message: compiledMessage,
        signatures: [new Uint8Array(64)],
        serialize: function() {
          return compiledMessage.serialize ? compiledMessage.serialize() : new Uint8Array();
        }
      };

      const signedTransaction = await provider.signTransaction(legacyTxMock);
      const signature = getSignatureFromTransaction(signedTransaction);

      await rpc.sendTransaction(signedTransaction, { encoding: 'base64', preflightCommitment: 'confirmed' }).send();
      txSignature.value = signature;

      const note = depositsStore.addNote(
        nullifierSecret,
        secret,
        Number(amountLamports),
        commitment,
        nullifierHash,
      );
      depositNote.value = note;

      const balanceResponse = await rpc.getBalance(typedUserAddress).send();
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
