<script setup lang="ts">
import WalletConnect from "@/components/WalletConnect.vue";
import DepositForm from "@/components/DepositForm.vue";
import WithdrawForm from "@/components/WithdrawForm.vue";
</script>

<template lang="pug">
.home
  header
    h1 Private Transfers on Solana
    p Zero-knowledge private pool for SOL transfers
    WalletConnect

  main
    .forms-grid
      .deposit-section
        DepositForm
      .withdraw-section
        WithdrawForm

  footer
    p Built with Noir, Sunspot, and Anchor
</template>

<style scoped lang="scss">
.home {
  min-height: 100vh;
  background: #f1f5f9;
  padding: 0 24px 40px;
}

header {
  text-align: center;
  padding: 40px 0 32px;

  h1 {
    font-size: 32px;
    font-weight: 700;
    color: #0f172a;
    margin: 0 0 8px 0;
  }

  p {
    font-size: 16px;
    color: #64748b;
    margin: 0 0 16px 0;
  }
}

.forms-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
  max-width: 1024px;
  margin: 0 auto;
}

.deposit-section,
.withdraw-section {
  display: flex;
  flex-direction: column;
}

footer {
  text-align: center;
  margin-top: 40px;
  padding-top: 24px;
  border-top: 1px solid #e2e8f0;

  p {
    font-size: 14px;
    color: #94a3b8;
    margin: 0;
  }
}

@media (max-width: 768px) {
  .forms-grid {
    grid-template-columns: 1fr;
  }

  header h1 {
    font-size: 24px;
  }
}
</style>
