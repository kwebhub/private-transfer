<script setup lang="ts">
import { computed } from "vue";
import type { DepositNote as DepositNoteType } from "@/services/storage";

const props = defineProps<{
  note: DepositNoteType;
}>();

const emit = defineEmits<{
  (e: "close"): void;
  (e: "saved"): void;
}>();

const amountSol = computed(() => {
  return (props.note.amount / 1_000_000_000).toFixed(4);
});

const copyAll = () => {
  const text =
    `=== DEPOSIT NOTE ===\n` +
    `Nullifier Secret: ${props.note.nullifierSecret}\n` +
    `Secret: ${props.note.secret}\n` +
    `Amount: ${amountSol.value} SOL\n` +
    `Commitment: ${props.note.commitment}\n` +
    `Nullifier Hash: ${props.note.nullifierHash}\n` +
    `=====================`;

  navigator.clipboard.writeText(text);
  alert("Deposit note copied to clipboard!");
};

const copyField = (value: string, label: string) => {
  navigator.clipboard.writeText(value);
  alert(`${label} copied to clipboard!`);
};

const handleSaved = () => {
  emit("saved");
};

const handleClose = () => {
  emit("close");
};
</script>

<template lang="pug">
.note-overlay(@click.self="handleClose")
  .note-modal
    .note-header
      h3 ⚠️ SAVE THIS DEPOSIT NOTE
      button.close-btn(@click="handleClose") ×

    .note-body
      p.warning
        | This note contains the secrets needed to withdraw your funds.
        | If you lose it, you will lose access to your deposited SOL.
        | Store it securely!

      .note-field
        .field-label Nullifier Secret
        .field-value
          span {{ note.nullifierSecret }}
          button.copy-btn(@click="copyField(note.nullifierSecret, 'Nullifier Secret')") Copy

      .note-field
        .field-label Secret
        .field-value
          span {{ note.secret }}
          button.copy-btn(@click="copyField(note.secret, 'Secret')") Copy

      .note-field
        .field-label Amount
        .field-value
          span {{ amountSol }} SOL
          button.copy-btn(@click="copyField(amountSol, 'Amount')") Copy

      .note-field
        .field-label Commitment
        .field-value
          span {{ note.commitment }}
          button.copy-btn(@click="copyField(note.commitment, 'Commitment')") Copy

      .note-field
        .field-label Nullifier Hash
        .field-value
          span {{ note.nullifierHash }}
          button.copy-btn(@click="copyField(note.nullifierHash, 'Nullifier Hash')") Copy

    .note-actions
      button.primary-btn(@click="copyAll") Copy All
      button.success-btn(@click="handleSaved") I've Saved It
</template>

<style scoped lang="scss">
.note-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.note-modal {
  background: white;
  border-radius: 16px;
  max-width: 560px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.note-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #e2e8f0;

  h3 {
    font-size: 18px;
    font-weight: 600;
    color: #ef4444;
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 28px;
    color: #94a3b8;
    cursor: pointer;
    line-height: 1;
    padding: 0 4px;

    &:hover {
      color: #0f172a;
    }
  }
}

.note-body {
  padding: 24px;

  .warning {
    background: #fef2f2;
    border: 1px solid #fecaca;
    border-radius: 8px;
    padding: 12px 16px;
    font-size: 14px;
    color: #991b1b;
    margin: 0 0 20px 0;
    line-height: 1.5;
  }
}

.note-field {
  margin-bottom: 16px;

  .field-label {
    font-size: 12px;
    font-weight: 600;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .field-value {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #f8fafc;
    border-radius: 6px;
    padding: 8px 12px;
    border: 1px solid #e2e8f0;

    span {
      font-family: monospace;
      font-size: 13px;
      color: #0f172a;
      word-break: break-all;
      flex: 1;
    }

    .copy-btn {
      background: white;
      border: 1px solid #e2e8f0;
      border-radius: 4px;
      padding: 4px 10px;
      font-size: 11px;
      color: #64748b;
      cursor: pointer;
      white-space: nowrap;

      &:hover {
        background: #f1f5f9;
        color: #0f172a;
      }
    }
  }
}

.note-actions {
  display: flex;
  gap: 12px;
  padding: 16px 24px 24px;
  border-top: 1px solid #e2e8f0;

  button {
    flex: 1;
    padding: 12px;
    border: none;
    border-radius: 8px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s;
  }

  .primary-btn {
    background: #4f46e5;
    color: white;

    &:hover {
      background: #4338ca;
    }
  }

  .success-btn {
    background: #22c55e;
    color: white;

    &:hover {
      background: #16a34a;
    }
  }
}
</style>
