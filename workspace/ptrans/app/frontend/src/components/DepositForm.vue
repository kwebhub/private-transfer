<script setup lang="ts">
import { ref, computed } from "vue";
import { useDeposit } from "@/composables/useDeposit";
import { useWallet } from "@/composables/useWallet";

const { isConnected } = useWallet();
const { loading, error, txSignature, depositNote, deposit, reset, MIN_DEPOSIT } = useDeposit();

const amount = ref<number>(0.01);
const showNote = ref(false);

const isValid = computed(() => {
  return amount.value >= MIN_DEPOSIT && isConnected;
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
        :href="`https://explorer.solana.com/tx/${txSignature}?cluster=devnet`"
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
  margin-top: 8px;

  &:hover:not(:disabled) {
    background: #4338ca;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  &.loading {
    opacity: 0.7;
  }
}

.error {
  color: #ef4444;
  font-size: 14px;
  margin-top: 8px;
}

.note-container h3 {
  font-size: 16px;
  font-weight: 600;
  color: #ef4444;
  margin: 0 0 16px 0;
}

.note-box {
  background: #f8fafc;
  border-radius: 8px;
  padding: 16px;
  border: 1px solid #e2e8f0;
  font-size: 13px;
  word-break: break-all;
}

.note-row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 0;
  border-bottom: 1px solid #e2e8f0;

  &:last-child {
    border-bottom: none;
  }

  .label {
    font-weight: 500;
    color: #64748b;
    font-size: 12px;
  }

  .value {
    font-family: monospace;
    color: #0f172a;
    font-size: 13px;
  }
}

.note-actions {
  display: flex;
  gap: 8px;
  margin-top: 16px;
}

.note-btn {
  flex: 1;
  padding: 10px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  background: white;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: #f1f5f9;
    border-color: #4f46e5;
  }
}

.tx-info {
  margin-top: 16px;
  font-size: 13px;
  color: #64748b;

  a {
    color: #4f46e5;
    text-decoration: none;
    font-family: monospace;
    margin-left: 8px;

    &:hover {
      text-decoration: underline;
    }
  }
}
</style>
