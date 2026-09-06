import {
  createSolanaRpc,
  createKeyPairFromPrivateKeyBytes,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstruction,
  pipe,
  signTransactionMessageWithSigners,
  getSignatureFromTransaction,
  createSignerFromKeyPair,
  getBase64EncodedWireTransaction
} from "@solana/kit";

import { getPoolInstructionAsync } from "../generated/instructions/pool";

async function main() {
  const rpc = createSolanaRpc("https://api.devnet.solana.com");
  const secretKeyString = "[190,24,234,189,169,43,13,122,30,236,240,108,176,11,45,226,128,226,118,46,153,119,207,226,45,32,99,198,64,109,134,79,53,32,72,105,150,170,238,106,251,92,9,194,51,214,243,56,112,246,146,210,1,134,65,85,40,164,106,208,237,130,95,89]";

  const parsedNumbersArray = JSON.parse(secretKeyString);
  const privateKeyBytes = new Uint8Array(parsedNumbersArray.slice(0, 32));

  const keyPair = await createKeyPairFromPrivateKeyBytes(privateKeyBytes);
  const authoritySigner = await createSignerFromKeyPair(keyPair);
  const authorityAddress = authoritySigner.address;

  console.log("Wallet Loaded (Authority):", authorityAddress);
  console.log("Initializing pool in devnet block ledger...");

  const poolInstruction = await getPoolInstructionAsync({
    authority: authoritySigner
  });

  const { value: latestBlockhash } = await rpc.getLatestBlockhash({ commitment: 'confirmed' }).send();

  const transactionMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (m) => setTransactionMessageFeePayer(authorityAddress, m),
    (m) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, m),
    (m) => appendTransactionMessageInstruction(poolInstruction, m)
  );

  const signedTransaction = await signTransactionMessageWithSigners(transactionMessage, [authoritySigner]);
  const signature = getSignatureFromTransaction(signedTransaction);
  const wireTransactionBase64 = getBase64EncodedWireTransaction(signedTransaction);

  await rpc.sendTransaction(wireTransactionBase64, { encoding: 'base64', preflightCommitment: 'confirmed' }).send();

  console.log("ПУЛ УСПЕШНО ИНИЦИАЛИЗИРОВАН В БЛОКЧЕЙНЕ!");
  console.log("Сигнатура транзакции:", signature);
}

main().catch(console.error);
