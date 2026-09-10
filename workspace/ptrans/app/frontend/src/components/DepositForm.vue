<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { storeToRefs } from "pinia";
import { useWalletStore } from "@/stores/wallet";
import { useDeposit } from "@/composables/useDeposit";
import { usePool } from "@/composables/usePool";

const walletStore = useWalletStore();
const { isConnected } = storeToRefs(walletStore);

const { loading, error, txSignature, depositNote, deposit, reset, MIN_DEPOSIT } = useDeposit();
const { poolInfo, loading: poolLoading, initializing, fetchPoolInfo, initializePool } = usePool();

const amount = ref<number>(MIN_DEPOSIT);
const showNote = ref(false);

// Проверяем, инициализирован ли пул
const isPoolInitialized = computed(() => {
  return poolInfo.value?.isInitialized === true;
});

// Проверяем валидность формы депозита
const isValid = computed(() => {
  return amount.value >= MIN_DEPOSIT && isConnected.value && isPoolInitialized.value;
});

// Обновляем информацию о пуле при подключении кошелька
watch(isConnected, (connected) => {
  if (connected) {
    fetchPoolInfo();
  }
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

const handleInitializePool = async () => {
  try {
    await initializePool();
  } catch (err) {
    console.error("Initialize pool error:", err);
  }
};

const handleReset = () => {
  reset();
  showNote.value = false;
  amount.value = MIN_DEPOSIT;
};

const copyNote = () => {
  if (!depositNote.value) return;
  const text = `Nullifier Secret: ${depositNote.value.nullifierSecret}\nSecret: ${depositNote.value.secret}\nAmount: ${depositNote.value.amount / 1_000_000_000} SOL\nCommitment: ${depositNote.value.commitment}`;
  navigator.clipboard.writeText(text);
  alert("Deposit note copied to clipboard!");
};

// Форматирование адреса для отображения
const formatAddress = (address: string) => {
  if (!address) return "";
  return `${address.slice(0, 6)}...${address.slice(-6)}`;
};

const copiedAddress = ref(false);

const copyPoolAddress = async () => {
  if (!poolInfo.value?.address) return;
  try {
    await navigator.clipboard.writeText(poolInfo.value.address);
    copiedAddress.value = true;
    setTimeout(() => {
      copiedAddress.value = false;
    }, 2000);
  } catch (err) {
    console.error("Failed to copy:", err);
  }
};
</script>

<template lang="pug">
.deposit-container
  h2 Deposit SOL

  // Статус пула
  .pool-status
    template(v-if="poolLoading")
      .status-loading
        span 🔄 Loading pool info...
    template(v-else-if="isPoolInitialized && poolInfo")
      .pool-initialized
        .status-row
          span.status-icon ✅
          span Pool initialized
        .address-row
          span.label Address:
          span.address-value {{ formatAddress(poolInfo.address) }}
          button.copy-btn(
            @click="copyPoolAddress"
            :title="copiedAddress ? 'Copied!' : 'Copy address'"
          ) {{ copiedAddress ? '✅' : '📋' }}
        .stats-row(v-if="poolInfo.totalDeposits > 0 || poolInfo.nextLeafIndex > 0")
          span Deposits: {{ poolInfo.totalDeposits }} SOL
          span Leafs: {{ poolInfo.nextLeafIndex }}
    template(v-else)
      .pool-not-initialized
        .warning-row
          span.status-icon ⚠️
          span Pool not initialized
        .action-row
          button.init-btn(
            @click="handleInitializePool"
            :disabled="!isConnected || initializing"
          )
            span(v-if="initializing") Initializing...
            span(v-else) Initialize Pool
          .hint(v-if="!isConnected") Connect wallet first

  // Форма депозита
  .form-group(v-if="!showNote")
    label Amount (SOL)
    input(
      type="number"
      v-model="amount"
      :disabled="loading || !isConnected || !isPoolInitialized"
      :step="MIN_DEPOSIT"
      :min="MIN_DEPOSIT"
    )
    .hint Minimum: {{ MIN_DEPOSIT }} SOL

    button.deposit-btn(
      @click="handleDeposit"
      :disabled="!isValid || loading"
      :class="{ loading }"
    )
      span(v-if="loading") Processing...
      span(v-else-if="!isPoolInitialized") Pool not initialized
      span(v-else) Deposit

    .error(v-if="error") {{ error }}

  // Deposit Note (после успешного депозита)
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
      button.note-btn.primary(@click="handleReset") Done, I've Saved It

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

// Статус пула
.pool-status {
  margin-bottom: 16px;
  padding: 12px 16px;
  background: #f8fafc;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}

.status-loading {
  color: #64748b;
  font-size: 13px;
}

.pool-initialized {
  display: flex;
  flex-direction: column;
  gap: 6px;

  .status-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 500;
    color: #065f46;

    .status-icon {
      font-size: 16px;
    }
  }

  .address-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: #475569;

    .label {
      color: #94a3b8;
    }

    .address-value {
      font-family: monospace;
      color: #0f172a;
      background: #f1f5f9;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 12px;
    }

    .copy-btn {
      background: none;
      border: none;
      cursor: pointer;
      font-size: 14px;
      padding: 2px 4px;
      border-radius: 4px;
      transition: background 0.2s;

      &:hover {
        background: #e2e8f0;
      }
    }
  }

  .stats-row {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: #64748b;
    margin-top: 4px;

    span {
      background: #f1f5f9;
      padding: 2px 8px;
      border-radius: 4px;
    }
  }
}

.pool-not-initialized {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;

  .warning-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 500;
    color: #ea580c;

    .status-icon {
      font-size: 16px;
    }
  }

  .action-row {
    display: flex;
    align-items: center;
    gap: 8px;

    .init-btn {
      background: #4f46e5;
      color: white;
      border: none;
      border-radius: 6px;
      padding: 6px 16px;
      font-size: 13px;
      font-weight: 500;
      cursor: pointer;
      white-space: nowrap;
      transition: all 0.2s;

      &:hover:not(:disabled) {
        background: #4338ca;
        transform: scale(1.02);
      }

      &:disabled {
        background: #cbd5e1;
        color: #94a3b8;
        cursor: not-allowed;
        transform: none;
      }
    }

    .hint {
      font-size: 12px;
      color: #94a3b8;
    }
  }
}

// Форма депозита
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
    transform: scale(1.02);
  }

  &:disabled {
    background: #cbd5e1;
    color: #94a3b8;
    cursor: not-allowed;
    transform: none;
  }

  &.loading {
    opacity: 0.7;
    cursor: wait;
  }
}

.error {
  color: #dc2626;
  font-size: 13px;
  margin-top: 4px;
}

// Note container
.note-container {
  display: flex;
  flex-direction: column;
  gap: 16px;

  h3 {
    color: #ea580c;
    font-size: 16px;
    margin: 0;
  }
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

  .label {
    color: #64748b;
    font-weight: 500;
  }

  .value {
    font-family: monospace;
    color: #0f172a;
    word-break: break-all;
    font-size: 12px;
  }
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
  transition: all 0.2s;

  &:hover {
    background: #f8fafc;
  }

  &.primary {
    background: #4f46e5;
    color: white;
    border-color: #4f46e5;

    &:hover {
      background: #4338ca;
    }
  }
}

.tx-info {
  margin-top: 8px;
  font-size: 13px;
  color: #64748b;

  a {
    color: #4f46e5;
    text-decoration: none;
    margin-left: 4px;

    &:hover {
      text-decoration: underline;
    }
  }
}
</style>
