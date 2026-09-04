<script setup lang="ts">
import { ref, computed } from "vue";
import { useWithdraw } from "@/composables/useWithdraw";
import { useWallet } from "@/composables/useWallet";
import { useDepositsStore } from "@/stores/deposits";

const { isConnected } = useWallet();
const { loading, error, txSignature, proofGenerating, withdraw, reset } = useWithdraw();
const depositsStore = useDepositsStore();

const selectedNoteId = ref<string>("");
const recipientAddress = ref<string>("");
const showNoteInput = ref(false);

const unusedNotes = computed(() => depositsStore.unusedNotes);

const selectedNote = computed(() => {
  if (!selectedNoteId.value) return null;
  return depositsStore.notes.find((n) => n.id === selectedNoteId.value) || null;
});

const isValid = computed(() => {
  return (
    selectedNote.value !== null &&
    recipientAddress.value.length > 0 &&
    isConnected &&
    !loading.value
  );
});

const handleWithdraw = async () => {
  if (!isValid.value || !selectedNote.value) return;

  const amountSol = selectedNote.value.amount / 1_000_000_000;

  try {
    await withdraw(
      selectedNote.value.nullifierSecret,
      selectedNote.value.secret,
      amountSol,
      recipientAddress.value,
    );
  } catch (err) {
    console.error("Withdraw error:", err);
  }
};

const handleReset = () => {
  reset();
  selectedNoteId.value = "";
  recipientAddress.value = "";
};

const copyTx = () => {
  if (!txSignature.value) return;
  navigator.clipboard.writeText(txSignature.value);
  alert("Transaction signature copied!");
};

const formatAddress = (addr: string) => {
  if (!addr) return "";
  return `${addr.slice(0, 4)}...${addr.slice(-4)}`;
};
</script>

<template lang="pug">
.withdraw-container
  h2 Withdraw SOL

  .form-group
    label Select Deposit Note
    select(v-model="selectedNoteId" :disabled="loading")
      option(value="") Select a note...
      option(
        v-for="note in unusedNotes"
        :key="note.id"
        :value="note.id"
      )
        | {{ formatAddress(note.nullifierSecret) }} - {{ (note.amount / 1_000_000_000).toFixed(3) }} SOL

  .note-preview(v-if="selectedNote")
    .preview-row
      span.label Amount:
      span.value {{ (selectedNote.amount / 1_000_000_000).toFixed(4) }} SOL
    .preview-row
      span.label Commitment:
      span.value {{ selectedNote.commitment.slice(0, 16) }}...

  .form-group
    label Recipient Address
    input(
      type="text"
      v-model="recipientAddress"
      :disabled="loading"
      placeholder="Enter Solana address"
    )

  button.withdraw-btn(
    @click="handleWithdraw"
    :disabled="!isValid"
    :class="{ loading }"
  )
    span(v-if="loading && proofGenerating") Generating proof...
    span(v-else-if="loading") Processing...
    span(v-else) Withdraw

  .error(v-if="error") {{ error }}

  .tx-info(v-if="txSignature")
    span Transaction:
    a(
      :href="`https://explorer.solana.com/tx/${txSignature}?cluster=devnet`"
      target="_blank"
    ) {{ txSignature.slice(0, 8) }}...{{ txSignature.slice(-8) }}
    button.copy-btn(@click="copyTx") Copy

  .hint(v-if="!unusedNotes.length && !loading")
    | No unused deposits found. Make a deposit first.
</template>

<style scoped lang="scss">
.withdraw-container {
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
  margin-bottom: 16px;
}

label {
  font-size: 14px;
  font-weight: 500;
  color: #334155;
}

select,
input {
  padding: 10px 12px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  font-size: 14px;
  transition: border-color 0.2s;
  background: white;

  &:focus {
    outline: none;
    border-color: #4f46e5;
  }

  &:disabled {
    background: #f1f5f9;
    cursor: not-allowed;
  }
}

.note-preview {
  background: #f8fafc;
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 16px;
  border: 1px solid #e2e8f0;
}

.preview-row {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  font-size: 14px;

  .label {
    color: #64748b;
  }

  .value {
    font-weight: 500;
    color: #0f172a;
    font-family: monospace;
  }
}

.withdraw-btn {
  background: #ef4444;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 12px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  width: 100%;

  &:hover:not(:disabled) {
    background: #dc2626;
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
  margin-top: 12px;
}

.tx-info {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  font-size: 13px;
  color: #64748b;

  a {
    color: #4f46e5;
    text-decoration: none;
    font-family: monospace;

    &:hover {
      text-decoration: underline;
    }
  }
}

.copy-btn {
  background: transparent;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 12px;
  color: #64748b;
  cursor: pointer;

  &:hover {
    background: #f1f5f9;
  }
}

.hint {
  margin-top: 16px;
  font-size: 14px;
  color: #64748b;
  text-align: center;
  padding: 16px;
  background: #f8fafc;
  border-radius: 8px;
}
</style>
