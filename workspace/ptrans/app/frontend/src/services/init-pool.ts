import {
  createSolanaRpc,
  address as solanaAddress,
  createKeyPairFromPrivateKeyBytes,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  pipe,
  signTransactionMessageWithSigners,
  getSignatureFromTransaction,
  createSignerFromKeyPair
} from "@solana/kit";
import { getPoolInstructionAsync } from "@/generated/instructions";

async function main() {
  const RPC_URL = process.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";
  const rpc = createSolanaRpc(RPC_URL);

  // 1. Извлекаем приватный ключ из переменных окружения
  const secretKeyArray = JSON.parse(process.env.WALLET_SECRET || "[]");
  if (secretKeyArray.length === 0) {
    throw new Error("WALLET_SECRET env variable is missing or empty");
  }

  // В Web3.js v2 создание пары ключей и подписывающего Signer-объекта разделено
  const keyPair = await createKeyPairFromPrivateKeyBytes(new Uint8Array(secretKeyArray));
  const authoritySigner = await createSignerFromKeyPair(keyPair);
  const authorityAddress = solanaAddress(keyPair.publicKey);

  console.log("Authority wallet loaded:", authorityAddress);
  console.log("Initializing pool...");

  // 2. Генерируем инструкцию инициализации пула через Codama
  // Функция автоматически вычислит все PDA контракта, если они размечены в IDL
  const poolInstruction = await getPoolInstructionAsync({
    // Если в вашем методе pool() требуются специфичные аккаунты, укажите их здесь.
    // Например, если требуется передать подписывающего админа:
    // user: authoritySigner.address,
  });

  // 3. Получаем свежий блокхеш
  const { value: latestBlockhash } = await rpc.getLatestBlockhash({ commitment: 'confirmed' }).send();

  // 4. Компонуем транзакцию с помощью pipe
  const transactionMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayer(authorityAddress, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstruction(poolInstruction, m)
  );

  // 5. Подписываем транзакцию нативным криптографическим ключом Node.js (работает без полифилов)
  const signedTransaction = await signTransactionMessageWithSigners(transactionMessage);
  const signature = getSignatureFromTransaction(signedTransaction);

  // 6. Отправляем в сеть и ждем процессинга
  await rpc.sendTransaction(signedTransaction, { encoding: 'base64', preflightCommitment: 'confirmed' }).send();

  console.log("🚀 Pool successfully initialized!");
  console.log("Transaction Signature:", signature);
}

main().catch(console.error);
