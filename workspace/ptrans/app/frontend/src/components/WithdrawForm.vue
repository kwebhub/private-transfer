<script setup lang="ts">
import { ref, computed } from "vue";
import { storeToRefs } from "pinia";
import { useWalletStore } from "@/stores/wallet";
import { useWithdraw } from "@/composables/useWithdraw";

const walletStore = useWalletStore();
const { isConnected } = storeToRefs(walletStore);

const { loading, error, txSignature, proofGenerating, withdraw, reset } = useWithdraw();

const nullifierNote = ref("");
const recipient = ref("");
const amount = ref(0.01);

const isValid = computed(() => {
  return nullifierNote.value.trim().length > 0 && recipient.value.trim().length > 0 && isConnected.value;
});

const handleWithdraw = async () => {
  if (!isValid.value) return;

  try {
    // Парсим заметку на секреты (или передаем как есть в зависимости от вашей архитектуры бэкенда)
    // Предполагаем, что ваша форма принимает nullifierSecret и secret раздельно или парсит общую строку
    const lines = nullifierNote.value.split("\n");
    const nullifierSecretHex = lines[0]?.replace("Nullifier Secret: ", "").trim() || "";
    const secretHex = lines[1]?.replace("Secret: ", "").trim() || "";

    await withdraw(nullifierSecretHex, secretHex, amount.value, recipient.value);
  } catch (err) {
    console.error("Withdraw execution error:", err);
  }
};
</script>

<template lang="pug">
// Шаблон вашей формы вывода, где кнопка привязана к :disabled="!isValid || loading"
.withdraw-container
  h2 Withdraw SOL
  .form-group
    label Deposit Note (Paste full note text)
    textarea(v-model="nullifierNote" :disabled="loading || !isConnected" rows="4" placeholder="Paste your saved note here...")
  .form-group
    label Recipient Address
    input(type="text" v-model="recipient" :disabled="loading || !isConnected" placeholder="Solana Base58 Address")

  button.withdraw-btn(@click="handleWithdraw" :disabled="!isValid || loading")
    span(v-if="proofGenerating") Generating ZK Proof...
    span(v-else-if="loading") Processing Tx...
    span(v-else) Withdraw

  .error(v-if="error") {{ error }}
  .success(v-if="txSignature") Success! Tx: {{ txSignature.slice(0,8) }}...
</template>

<style scoped lang="scss">
/* Добавьте стили аналогично форме депозита */
.withdraw-container { background: white; border-radius: 12px; padding: 24px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); max-width: 480px; margin: 20px auto; display: flex; flex-direction: column; gap: 16px; }
.form-group { display: flex; flex-direction: column; gap: 6px; label { font-size: 14px; font-weight: 500; color: #334155; } }
textarea, input { padding: 10px; border: 1px solid #e2e8f0; border-radius: 8px; font-size: 15px; font-family: monospace; }
.withdraw-btn { background: #4f46e5; color: white; border: none; border-radius: 8px; padding: 12px; font-weight: 600; cursor: pointer; transition: background 0.2s; &:disabled { background: #cbd5e1; color: #94a3b8; cursor: not-allowed; } }
.error { color: #ef4444; font-size: 14px; }
.success { color: #16a34a; font-size: 14px; }
</style>
