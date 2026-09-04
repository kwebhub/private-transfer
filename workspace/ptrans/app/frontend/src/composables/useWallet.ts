import { onMounted } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { createSolanaRpc, address as solanaAddress } from "@solana/kit";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://solana.com";

export function useWallet() {
  const store = useWalletStore();
  const rpc = createSolanaRpc(RPC_URL);

  function getProvider() {
    if (typeof window === "undefined") return null;
    return (window as any).phantom?.solana || (window as any).solana || (window as any).solflare;
  }

  function extractAddress(response: any, provider: any): string {
    if (provider && provider.publicKey) return provider.publicKey.toString();
    if (response && response.publicKey) return response.publicKey.toString();
    if (typeof response === "string") return response;
    if (response && typeof response.toString === "function" && response !== true) return response.toString();
    return "";
  }

  async function refreshBalance(): Promise<void> {
    if (!store.walletAddress) return;
    try {
      const response = await rpc.getBalance(solanaAddress(store.walletAddress)).send();
      store.setBalance(response.value);
    } catch (error) {
      console.error("Failed to refresh balance:", error);
    }
  }

  async function connect(): Promise<void> {
    if (store.isConnecting) return;
    store.setConnecting(true);

    try {
      const provider = getProvider();
      if (!provider) {
        throw new Error("Solana wallet extension not found. Please install Phantom or Solflare.");
      }

      const response = await provider.connect();
      const addressStr = extractAddress(response, provider);

      if (!addressStr || addressStr.length < 32 || addressStr.length > 44) {
        throw new Error(`Invalid address extracted from wallet: ${addressStr}`);
      }

      store.setWallet(addressStr, "Solana Extension");
      await refreshBalance();

      if (typeof provider.on === "function") {
        provider.on("accountChanged", async (newPublicKey: any) => {
          if (newPublicKey) {
            store.setWallet(newPublicKey.toString(), "Solana Extension");
            await refreshBalance();
          } else {
            disconnect();
          }
        });
      }

    } catch (error) {
      console.error("Failed to connect wallet:", error);
      alert(error instanceof Error ? error.message : "Connection failed");
    } finally {
      store.setConnecting(false);
    }
  }

  async function disconnect(): Promise<void> {
    try {
      const provider = getProvider();
      if (provider && typeof provider.disconnect === "function") {
        await provider.disconnect();
      }
    } catch (error) {
      console.error("Failed to disconnect wallet:", error);
    } finally {
      store.disconnect();
    }
  }

  function getActiveWalletInterface() {
    return null;
  }

  onMounted(async () => {
    const provider = getProvider();
    if (provider?.isConnected && provider?.publicKey) {
      const addressStr = provider.publicKey.toString();
      store.setWallet(addressStr, "Solana Extension");
      await refreshBalance();
    }
  });

  return {
    rpc,
    getActiveWalletInterface,
    connect,
    disconnect,
    refreshBalance,
  };
}
