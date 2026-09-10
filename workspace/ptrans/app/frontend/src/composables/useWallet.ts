import { onMounted } from "vue";
import { useWalletStore } from "@/stores/wallet";
import { createSolanaRpc, address as solanaAddress } from "@solana/kit";

const RPC_URL = import.meta.env.VITE_SOLANA_RPC_URL || "https://api.devnet.solana.com";

export function useWallet() {
  const store = useWalletStore();
  const rpc = createSolanaRpc(RPC_URL);

  function getProvider() {
    if (typeof window === "undefined") return { provider: null, name: "" };

    if ((window as any).phantom?.solana) {
      return { provider: (window as any).phantom.solana, name: "Phantom" };
    }
    if ((window as any).solflare) {
      return { provider: (window as any).solflare, name: "Solflare" };
    }
    if ((window as any).solana) {
      return { provider: (window as any).solana, name: "Solana Wallet" };
    }

    return { provider: null, name: "" };
  }

  function extractAddress(response: any, provider: any): string {
    if (provider && provider.publicKey) return provider.publicKey.toString();
    if (response && response.publicKey) return response.publicKey.toString();
    if (typeof response === "string") return response;
    if (response && typeof response.toString === "function" && response !== true)
      return response.toString();
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
      const { provider, name } = getProvider();
      if (!provider) {
        throw new Error("Solana wallet extension not found. Please install Phantom or Solflare.");
      }

      const response = await provider.connect();
      const addressStr = extractAddress(response, provider);

      if (!addressStr || addressStr.length < 32 || addressStr.length > 44) {
        throw new Error(`Invalid address extracted from wallet: ${addressStr}`);
      }

      store.setWallet(addressStr, name);
      await refreshBalance();

      if (typeof provider.on === "function") {
        provider.on("accountChanged", async (newPublicKey: any) => {
          if (newPublicKey) {
            store.setWallet(newPublicKey.toString(), name);
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
      const { provider } = getProvider();
      if (provider && typeof provider.disconnect === "function") {
        await provider.disconnect();
      }
    } catch (error) {
      console.error("Failed to disconnect wallet:", error);
    } finally {
      store.disconnect();
    }
  }

  onMounted(async () => {
    const { provider, name } = getProvider();
    if (provider?.isConnected && provider?.publicKey) {
      store.setWallet(provider.publicKey.toString(), name);
      await refreshBalance();
    }
  });

  return {
    rpc,
    connect,
    disconnect,
    refreshBalance,
  };
}
