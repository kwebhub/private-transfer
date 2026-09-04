<script setup lang="ts">
import { onMounted } from "vue";
import { usePool } from "@/composables/usePool";
import WalletConnect from "@/components/WalletConnect.vue";

const { loading, error, poolInfo, fetchPoolInfo } = usePool();

onMounted(() => {
  fetchPoolInfo();
});

const formatSol = (lamports: number) => {
  return (lamports / 1_000_000_000).toFixed(4);
};

const formatAddress = (addr: string) => {
  if (!addr) return "";
  return `${addr.slice(0, 6)}...${addr.slice(-6)}`;
};
</script>

<template lang="pug">
.pool-page
  header
    h1 Pool Information
    WalletConnect

  .pool-container
    .loading(v-if="loading") Loading pool information...

    .error(v-else-if="error") {{ error }}

    .pool-card(v-else-if="poolInfo")
      .info-grid
        .info-item
          .label Pool Address
          .value
            span {{ formatAddress(poolInfo.address) }}
            a(
              :href="`https://explorer.solana.com/address/${poolInfo.address}?cluster=devnet`"
              target="_blank"
            ) 🔗

        .info-item
          .label Vault Address
          .value
            span {{ formatAddress(poolInfo.vaultAddress) }}
            a(
              :href="`https://explorer.solana.com/address/${poolInfo.vaultAddress}?cluster=devnet`"
              target="_blank"
            ) 🔗

        .info-item
          .label Vault Balance
          .value {{ poolInfo.vaultBalanceSol }} SOL
          .sub {{ poolInfo.vaultBalance }} lamports

        .info-item
          .label Total Deposits
          .value {{ poolInfo.totalDeposits }}

        .info-item
          .label Next Leaf Index
          .value {{ poolInfo.nextLeafIndex }}

        .info-item
          .label Nullifiers Used
          .value {{ poolInfo.nullifiersCount }}

        .info-item.full
          .label Current Merkle Root
          .value.root {{ poolInfo.currentRoot }}

    .empty-state(v-else)
      p No pool information available.
      p.hint Make sure the pool is initialized.

  .actions
    button.refresh-btn(@click="fetchPoolInfo" :disabled="loading")
      span(v-if="loading") Loading...
      span(v-else) Refresh
</template>

<style scoped lang="scss">
.pool-page {
  min-height: 100vh;
  background: #f1f5f9;
  padding: 0 24px 40px;
}

header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 0;

  h1 {
    font-size: 24px;
    font-weight: 700;
    color: #0f172a;
    margin: 0;
  }
}

.pool-container {
  max-width: 720px;
  margin: 0 auto;
}

.loading,
.error,
.empty-state {
  text-align: center;
  padding: 40px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.error {
  color: #ef4444;
}

.empty-state {
  color: #64748b;

  .hint {
    font-size: 14px;
    margin-top: 8px;
  }
}

.pool-card {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 16px;
  background: #f8fafc;
  border-radius: 8px;
  border: 1px solid #e2e8f0;

  .label {
    font-size: 12px;
    font-weight: 600;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .value {
    font-size: 16px;
    font-weight: 500;
    color: #0f172a;
    display: flex;
    align-items: center;
    gap: 8px;

    a {
      color: #4f46e5;
      text-decoration: none;
      font-size: 14px;

      &:hover {
        text-decoration: underline;
      }
    }
  }

  .sub {
    font-size: 12px;
    color: #94a3b8;
    font-family: monospace;
  }

  &.full {
    grid-column: 1 / -1;
  }

  .root {
    font-family: monospace;
    font-size: 14px;
    word-break: break-all;
  }
}

.actions {
  text-align: center;
  margin-top: 24px;
}

.refresh-btn {
  background: #4f46e5;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 10px 24px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;

  &:hover:not(:disabled) {
    background: #4338ca;
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
}

@media (max-width: 600px) {
  .info-grid {
    grid-template-columns: 1fr;
  }

  header {
    flex-direction: column;
    gap: 16px;
    text-align: center;
  }
}
</style>
