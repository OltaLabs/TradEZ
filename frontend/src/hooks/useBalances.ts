import { useCallback, useEffect, useRef, useState } from "react";
import { ethers } from "ethers";
import { EIP1193Provider, getMetaMaskProvider } from "@/hooks/useWallet";

const ERC20_ABI = [
  "function balanceOf(address owner) view returns (uint256)",
  "function decimals() view returns (uint8)",
];

const INITIAL_BALANCES = {
  xtz: null as string | null,
  usdc: null as string | null,
};

const resolveProvider = (): EIP1193Provider | undefined => {
  if (typeof window === "undefined") {
    return undefined;
  }
  return getMetaMaskProvider() ?? window.ethereum;
};

type FetchOptions = {
  silent?: boolean;
};

export const useBalances = (account: string | null) => {
  const [balances, setBalances] = useState(INITIAL_BALANCES);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    return () => {
      mountedRef.current = false;
    };
  }, []);

  const fetchBalances = useCallback(
    async ({ silent = false }: FetchOptions = {}) => {
      if (!account) {
        if (!mountedRef.current) {
          return;
        }
        setBalances(INITIAL_BALANCES);
        setError(null);
        setLoading(false);
        return;
      }

      const providerSource = resolveProvider();
      if (!providerSource) {
        if (!mountedRef.current) {
          return;
        }
        setError("Wallet provider unavailable");
        setBalances(INITIAL_BALANCES);
        setLoading(false);
        return;
      }

      try {
        if (!silent && mountedRef.current) {
          setLoading(true);
        }

        setBalances({
          xtz: ethers.formatUnits(0, 18),
          usdc: ethers.formatUnits(0, 6),
        });
        setError(null);
      } catch (err: any) {
        if (!mountedRef.current) {
          return;
        }
        console.error("Failed to fetch balances:", err);
        setError(err?.message ?? "Unable to load balances");
      } finally {
        if (!mountedRef.current || silent) {
          return;
        }
        setLoading(false);
      }
    },
    [account]
  );

  useEffect(() => {
    let intervalId: number | null = null;

    fetchBalances();

    if (account) {
      intervalId = window.setInterval(() => {
        fetchBalances({ silent: true }).catch(() => undefined);
      }, 15000);
    }

    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, [account, fetchBalances]);

  return {
    xtz: balances.xtz,
    usdc: balances.usdc,
    loading,
    error,
    refresh: fetchBalances,
  };
};
