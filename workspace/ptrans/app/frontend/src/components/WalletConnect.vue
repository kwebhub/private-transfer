<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useWalletStore } from "@/stores/wallet";
import { useWallet } from "@/composables/useWallet";

const store = useWalletStore();
// storeToRefs — это официальный способ сохранить 100% реактивность при деструктуризации
const { isConnected, isConnecting, walletAddress, walletBalance } = storeToRefs(store);
const { connect, disconnect } = useWallet();

const shortAddress = computed(() => {
  if (!walletAddress.value) return "";
  return `${walletAddress.value.slice(0, 4)}...${walletAddress.value.slice(-4)}`;
});

const formattedBalance = computed(() => {
  const sol = Number(walletBalance.value) / 1_000_000_000;
  return sol.toFixed(4);
});

const handleConnect = async () => {
  try {
    await connect();
  } catch (err) {
    console.error("Connection error:", err);
  }
};

const handleDisconnect = async () => {
  try {
    await disconnect();
  } catch (err) {
    console.error("Disconnect error:", err);
  }
};
</script>

<template lang="pug">
.wallet-container
  // В Pug пишем чистые переменные без всяких .value! Vue развернет их сам
  template(v-if="!isConnected")
    button.wallet-connect(
      @click="handleConnect"
      :disabled="isConnecting"
      :class="{ loading: isConnecting }"
    )
      span(v-if="isConnecting") Connecting...
      span(v-else) Connect Wallet

  template(v-else)
    .wallet-info
      .wallet-address
        span.address-label Address:
        span.address-value {{ shortAddress }}
      .wallet-balance
        span.balance-label Balance:
        span.balance-value {{ formattedBalance }} SOL
      button.wallet-disconnect(@click="handleDisconnect") Disconnect
</template>

<style scoped lang="scss">
.wallet-container {
  display: flex;
  justify-content: flex-end;
  padding: 16px;
}

.wallet-connect {
  background: #4f46e5;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;

  &:hover:not(:disabled) {
    background: #4338ca;
    transform: translateY(-1px);
  }

  &:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  &.loading {
    opacity: 0.7;
  }
}

.wallet-info {
  display: flex;
  align-items: center;
  gap: 16px;
  background: #f8fafc;
  border-radius: 8px;
  padding: 8px 16px;
  border: 1px solid #e2e8f0;
}

.wallet-address {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.address-label {
  color: #64748b;
}

.address-value {
  font-family: monospace;
  font-weight: 500;
  color: #0f172a;
}

.wallet-balance {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  padding-left: 16px;
  border-left: 1px solid #e2e8f0;
}

.balance-label {
  color: #64748b;
}

.balance-value {
  font-weight: 600;
  color: #0f172a;
}

.wallet-disconnect {
  background: transparent;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
  padding: 4px 12px;
  font-size: 12px;
  color: #64748b;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: #f1f5f9;
    color: #ef4444;
    border-color: #ef4444;
  }
}
</style>
