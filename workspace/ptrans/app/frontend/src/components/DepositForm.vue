<script setup lang="ts">
import { ref, computed } from "vue";
import { storeToRefs } from "pinia";
import { useWalletStore } from "@/stores/wallet";
import { useDeposit } from "@/composables/useDeposit";

const walletStore = useWalletStore();
// Извлекаем реактивное состояние подключения напрямую из стора
const { isConnected } = storeToRefs(walletStore);

const { loading, error, txSignature, depositNote, deposit, reset, MIN_DEPOSIT } = useDeposit();

const amount = ref<number>(0.01);
const showNote = ref(false);

const isValid = computed(() => {
  return amount.value >= MIN_DEPOSIT && isConnected.value;
});

const handleDeposit = async () => {
  if (!isValid.value) return;

  try {
    await deposit(amount.value);
    showNote.value = true;
  } catch (err) {
    console.error("Deposit error:", err);
  }
};

const handleReset = () => {
  reset();
  showNote.value = false;
  amount.value = 0.01;
};

const copyNote = () => {
  if (!depositNote.value) return;
  const text = `Nullifier Secret: ${depositNote.value.nullifierSecret}\nSecret: ${depositNote.value.secret}\nAmount: ${depositNote.value.amount / 1_000_000_000} SOL\nCommitment: ${depositNote.value.commitment}`;
  navigator.clipboard.writeText(text);
  alert("Deposit note copied to clipboard!");
};
</script>

<template lang="pug">
.deposit-container
  h2 Deposit SOL

  .form-group(v-if="!showNote")
    label Amount (SOL)
    input(
      type="number"
      v-model="amount"
      :disabled="loading || !isConnected"
      step="0.001"
      min="0.001"
    )
    .hint Minimum: {{ MIN_DEPOSIT }} SOL

    button.deposit-btn(
      @click="handleDeposit"
      :disabled="!isValid || loading"
      :class="{ loading }"
    )
      span(v-if="loading") Processing...
      span(v-else) Deposit

    .error(v-if="error") {{ error }}

  // Deposit Note
  .note-container(v-else)
    h3 ⚠️ Save Your Deposit Note!
    .note-box
      .note-row
        span.label Nullifier Secret:
        span.value {{ depositNote?.nullifierSecret }}
      .note-row
        span.label Secret:
        span.value {{ depositNote?.secret }}
      .note-row
        span.label Amount:
        span.value {{ (depositNote?.amount || 0) / 1_000_000_000 }} SOL
      .note-row
        span.label Commitment:
        span.value {{ depositNote?.commitment }}

    .note-actions
      button.note-btn(@click="copyNote") Copy Note
      button.note-btn(@click="handleReset") Done, I've Saved It

    .tx-info(v-if="txSignature")
      span Transaction:
      a(
        :href="`https://solana.com{txSignature}?cluster=devnet`"
        target="_blank"
      ) {{ txSignature.slice(0, 8) }}...{{ txSignature.slice(-8) }}
</template>

<style scoped lang="scss">
.deposit-container {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  max-width: 480px;
  margin: 0 auto;
}

h2 {
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 20px 0;
  color: #0f172a;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  font-size: 14px;
  font-weight: 500;
  color: #334155;
}

input {
  padding: 10px 12px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  font-size: 16px;
  transition: border-color 0.2s;

  &:focus {
    outline: none;
    border-color: #4f46e5;
  }

  &:disabled {
    background: #f1f5f9;
    cursor: not-allowed;
  }
}

.hint {
  font-size: 12px;
  color: #64748b;
}

.deposit-btn {
  background: #4f46e5;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;

  &:hover:not(:disabled) {
    background: #4338ca;
  }

  &:disabled {
    background: #cbd5e1;
    color: #94a3b8;
    cursor: not-allowed;
  }
}

.note-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
  h3 { color: #ea580c; font-size: 16px; margin: 0; }
}

.note-box {
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.note-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 13px;
  .label { color: #64748b; font-weight: 500; }
  .value { font-family: monospace; color: #0f172a; word-break: break-all; }
}

.note-actions {
  display: flex;
  gap: 12px;
}

.note-btn {
  flex: 1;
  padding: 10px;
  border-radius: 6px;
  border: 1px solid #e2e8f0;
  background: white;
  cursor: pointer;
  font-weight: 500;
  transition: background 0.2s;
  &:hover { background: #f8fafc; }
  &:last-child { background: #4f46e5; color: white; border-color: #4f46e5; &:hover { background: #4338ca; } }
}

.tx-info {
  margin-top: 8px;
  font-size: 13px;
  color: #64748b;
  a { color: #4f46e5; text-decoration: none; margin-left: 4px; &:hover { text-decoration: underline; } }
}
</style>
