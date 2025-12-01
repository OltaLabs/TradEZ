import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, ReactNode } from "react";
import { ethers } from "ethers";
import { useToast } from "@/hooks/use-toast";

export type EIP1193Provider = {
  request: (args: { method: string; params?: unknown[] }) => Promise<any>;
  on?: (event: string, callback: (...args: any[]) => void) => void;
  removeAllListeners?: (event: string) => void;
  off?: (event: string, callback: (...args: any[]) => void) => void;
  isMetaMask?: boolean;
  providers?: EIP1193Provider[];
  _metamask?: {
    isUnlocked: () => Promise<boolean>;
  };
  selectedAddress?: string;
};

declare global {
  interface Window {
    ethereum?: EIP1193Provider;
  }
}

export const getMetaMaskProvider = (): EIP1193Provider | undefined => {
  if (typeof window === "undefined") return undefined;
  const { ethereum } = window;
  if (!ethereum) return undefined;
  if (Array.isArray(ethereum.providers) && ethereum.providers.length > 0) {
    return ethereum.providers.find((p) => p.isMetaMask) ?? ethereum.providers[0];
  }
  return ethereum.isMetaMask ? ethereum : undefined;
};

type WalletContextValue = {
  account: string | null;
  connecting: boolean;
  connectWallet: () => Promise<void>;
  disconnectWallet: () => void;
  signMessage: (message: string | Uint8Array) => Promise<string | null>;
};

const WalletContext = createContext<WalletContextValue | undefined>(undefined);

export const WalletProvider = ({ children }: { children: ReactNode }) => {
  const [account, setAccount] = useState<string | null>(null);
  const [connecting, setConnecting] = useState(false);
  const { toast } = useToast();
  const providerRef = useRef<EIP1193Provider | null>(null);
  const cleanupListeners = useRef<(() => void) | null>(null);

  const isProviderUnlocked = useCallback(async (provider: EIP1193Provider | null | undefined) => {
    if (!provider) {
      return false;
    }
    if (provider._metamask?.isUnlocked) {
      try {
        return await provider._metamask.isUnlocked();
      } catch (err) {
        console.warn("Unable to determine MetaMask lock state", err);
        return false;
      }
    }
    return true;
  }, []);

  const connectWallet = useCallback(async () => {
    const injectedProvider = getMetaMaskProvider();
    providerRef.current = injectedProvider ?? null;
    if (!injectedProvider) {
      toast({
        title: "MetaMask not found",
        description: "Please install MetaMask to use this application",
        variant: "destructive",
      });
      return;
    }

    try {
      setConnecting(true);
      const provider = new ethers.BrowserProvider((injectedProvider ??
        providerRef.current) as any);
      const accounts = await provider.send("eth_requestAccounts", []);
      if (accounts.length > 0) {
        setAccount(accounts[0]);
        toast({
          title: "Wallet connected",
          description: `Connected to ${accounts[0].slice(0, 6)}...${accounts[0].slice(-4)}`,
        });
      }
    } catch (error: any) {
      console.error("Error connecting wallet:", error);
      toast({
        title: "Connection failed",
        description: error.message || "Failed to connect wallet",
        variant: "destructive",
      });
    } finally {
      setConnecting(false);
    }
  }, [toast]);

  const signMessage = useCallback(
    async (message: string | Uint8Array): Promise<string | null> => {
      const injectedProvider = providerRef.current ?? getMetaMaskProvider();
      if (!account || !injectedProvider) {
        return null;
      }

      try {
        const provider = new ethers.BrowserProvider(injectedProvider as any);
        const signer = await provider.getSigner();
        const signature = await signer.signMessage(message);
        return signature;
      } catch (error: any) {
        console.error("Error signing message:", error);
        throw error;
      }
    },
    [account]
  );

  const syncAccountFromProvider = useCallback(
    async (
      provider: EIP1193Provider | null | undefined,
      accounts: string[] | null
    ) => {
      if (!provider) {
        setAccount(null);
        return;
      }
      const unlocked = await isProviderUnlocked(provider);
      const activeAccount = provider.selectedAddress ?? window.ethereum?.selectedAddress;
      if (!unlocked || !activeAccount) {
        setAccount(null);
        return;
      }
      const list = accounts ?? (await provider.request({ method: "eth_accounts" }));
      if (!Array.isArray(list) || list.length === 0) {
        setAccount(null);
        return;
      }
      const normalized = activeAccount.toLowerCase();
      if (list.some((a) => typeof a === "string" && a.toLowerCase() === normalized)) {
        setAccount(activeAccount);
      } else {
        setAccount(null);
      }
    },
    [isProviderUnlocked]
  );

  useEffect(() => {
    const injectedProvider = getMetaMaskProvider();
    providerRef.current = injectedProvider ?? null;
    if (injectedProvider) {
      syncAccountFromProvider(injectedProvider, null).catch(console.error);

      const handleAccountsChanged = async (accounts: string[]) => {
        await syncAccountFromProvider(providerRef.current, accounts);
      };

      const handleChainChanged = () => {
        window.location.reload();
      };

      injectedProvider.on?.("accountsChanged", handleAccountsChanged);
      injectedProvider.on?.("chainChanged", handleChainChanged);

      cleanupListeners.current = () => {
        injectedProvider.removeAllListeners?.("accountsChanged");
        injectedProvider.removeAllListeners?.("chainChanged");
        injectedProvider.off?.("accountsChanged", handleAccountsChanged);
        injectedProvider.off?.("chainChanged", handleChainChanged);
      };
    }

    return () => {
      cleanupListeners.current?.();
      cleanupListeners.current = null;
    };
  }, [syncAccountFromProvider]);

  const disconnectWallet = useCallback(() => {
    cleanupListeners.current?.();
    cleanupListeners.current = null;
    providerRef.current = null;
    setAccount(null);
    toast({
      title: "Wallet disconnected",
    });
  }, [toast]);

  const value = useMemo(
    () => ({
      account,
      connecting,
      connectWallet,
      disconnectWallet,
      signMessage,
    }),
    [account, connecting, connectWallet, disconnectWallet, signMessage]
  );

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
};

export const useWallet = () => {
  const context = useContext(WalletContext);
  if (!context) {
    throw new Error("useWallet must be used within a WalletProvider");
  }
  return context;
};
