import { defineStore } from "pinia";

export const useWalletStore = defineStore("wallet", {
  state: () => ({
    isConnected: false,
    isConnecting: false,
    walletAddress: null as string | null,
    walletBalance: 0n, // Используем bigint для лампортов
    activeWalletName: null as string | null,
  }),
  actions: {
    setConnecting(value: boolean) {
      this.isConnecting = value;
    },
    setWallet(address: string, walletName: string) {
      this.walletAddress = address;
      this.activeWalletName = walletName;
      this.isConnected = true;
    },
    setBalance(lamports: bigint) {
      this.walletBalance = lamports;
    },
    disconnect() {
      this.isConnected = false;
      this.walletAddress = null;
      this.walletBalance = 0n;
      this.activeWalletName = null;
    },
  },
});
