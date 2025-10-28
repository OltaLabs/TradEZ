import { useCallback, useMemo } from "react";

export type RpcCurrency = "USDC" | "XTZ";
export type RpcQty = number;
export type RpcPrice = number;
export type RpcSignatureInput = string | Uint8Array | number[];

export type RpcOrder = {
  side: "Bid" | "Ask";
  size: RpcQty;
  price: RpcPrice;
  nonce: number;
};

export type RpcCancelOrder = {
  order_id: number;
};

export type RpcFaucet = {
  amount: RpcQty;
};

export type RpcBalancesResult = Array<[RpcCurrency, RpcQty]>;
export type RpcOrderbookLevels = Array<[RpcPrice, RpcQty]>;
export type RpcOrderbookState = [RpcOrderbookLevels, RpcOrderbookLevels];
export type RpcUserOrder = {
  side: "Bid" | "Ask";
  ord_type: "Limit" | "Market";
  price: RpcPrice;
  qty: RpcQty;
  remaining: RpcQty;
  nonce: number;
};
export type RpcOrdersResult = Array<[number, RpcUserOrder]>;

const trimTrailingSlash = (value?: string) => value?.replace(/\/+$/, "");

const parseBody = async (response: Response) => {
  const text = await response.text();
  if (!text) {
    return null;
  }
  try {
    return JSON.parse(text);
  } catch {
    return text;
  }
};

const toByteArray = (input: RpcSignatureInput): number[] => {
  if (typeof input === "string") {
    const hex = input.startsWith("0x") ? input.slice(2) : input;
    if (hex.length % 2 !== 0) {
      throw new Error("Invalid hex signature.");
    }
    const bytes: number[] = [];
    for (let i = 0; i < hex.length; i += 2) {
      const value = Number.parseInt(hex.slice(i, i + 2), 16);
      if (Number.isNaN(value)) {
        throw new Error("Invalid hex signature.");
      }
      bytes.push(value);
    }
    return bytes;
  }
  if (input instanceof Uint8Array) {
    return Array.from(input);
  }
  return [...input];
};

type JsonRpcSuccess<T> = {
  jsonrpc: "2.0";
  id: number | string | null;
  result: T;
};

type JsonRpcError = {
  jsonrpc: "2.0";
  id: number | string | null;
  error: {
    code: number;
    message: string;
    data?: unknown;
  };
};

export const useTradezApi = () => {
  const apiBaseUrl = useMemo(
    () => trimTrailingSlash(import.meta.env.VITE_TRADEZ_API_URL as string | undefined),
    []
  );

  const callRpc = useCallback(
    async <T>(method: string, params: unknown[]) => {
      if (!apiBaseUrl) {
        throw new Error("Set VITE_TRADEZ_API_URL to enable backend requests.");
      }
      const response = await fetch(apiBaseUrl, {
        method: "POST",
        headers: {
          Accept: "application/json",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: Date.now(),
          method,
          params,
        }),
      });

      const payload = (await parseBody(response)) as
        | JsonRpcSuccess<T>
        | JsonRpcError
        | string
        | null;

      if (!response.ok) {
        const message =
          typeof payload === "string"
            ? payload
            : payload && "error" in (payload as any)
              ? (payload as JsonRpcError).error.message
              : `Request failed with status ${response.status}`;
        throw new Error(message);
      }

      if (!payload || typeof payload !== "object") {
        throw new Error("Invalid JSON-RPC response.");
      }

      if ("error" in payload) {
        throw new Error(payload.error.message);
      }

      return (payload as JsonRpcSuccess<T>).result;
    },
    [apiBaseUrl]
  );

  const sendOrder = useCallback(
    async (order: RpcOrder, signature: RpcSignatureInput) => {
      return callRpc<string>("send_order", [order, toByteArray(signature)]);
    },
    [callRpc]
  );

  const cancelOrder = useCallback(
    async (params: RpcCancelOrder, signature: RpcSignatureInput) => {
      return callRpc<string>("cancel_order", [params, toByteArray(signature)]);
    },
    [callRpc]
  );

  const faucet = useCallback(
    async (params: RpcFaucet, signature: RpcSignatureInput) => {
      return callRpc<string>("faucet", [params, toByteArray(signature)]);
    },
    [callRpc]
  );

  const getBalances = useCallback(
    async (address: string) => {
      return callRpc<RpcBalancesResult>("get_balances", [address]);
    },
    [callRpc]
  );

  const getOrders = useCallback(
    async (address: string) => {
      return callRpc<RpcOrdersResult>("get_orders", [address]);
    },
    [callRpc]
  );

  const getOrderbookState = useCallback(async () => {
    return callRpc<RpcOrderbookState>("get_orderbook_state", []);
  }, [callRpc]);

  return {
    apiUrl: apiBaseUrl,
    isApiConfigured: Boolean(apiBaseUrl),
    sendOrder,
    cancelOrder,
    faucet,
    getBalances,
    getOrders,
    getOrderbookState,
  };
};
