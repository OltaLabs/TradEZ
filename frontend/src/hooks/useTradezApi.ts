import { useCallback } from "react";

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
  currency: RpcCurrency;
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
export type RpcEvent =
  | {
      Placed: {
        user: unknown;
        id: number;
        side: "Bid" | "Ask";
        price: RpcPrice;
        qty: RpcQty;
      };
    }
  | {
      Trade: {
        maker_id: number;
        maker_user: unknown;
        taker_id: number;
        taker_user: unknown;
        price: RpcPrice;
        qty: RpcQty;
        origin_side: "Bid" | "Ask";
      };
    }
  | {
      Done: {
        user: unknown;
        id: number;
      };
    }
  | {
      Cancelled: {
        user: unknown;
        id: number;
        reason: string;
      };
    };

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

const API_BASE_URL = trimTrailingSlash(
  import.meta.env.VITE_TRADEZ_API_URL as string | undefined
);

const WS_BASE_URL = (() => {
  if (!API_BASE_URL) {
    return null;
  }
  try {
    const url = new URL(API_BASE_URL);
    url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
    return url.toString();
  } catch {
    console.error("Invalid API URL; websocket subscriptions disabled.");
    return null;
  }
})();

type SubscriptionListener = (payload: any) => void;

class SubscriptionManager {
  private socket: WebSocket | null = null;
  private listeners = new Map<string, Set<SubscriptionListener>>();
  private subscribedMethods = new Set<string>();
  private reconnectTimeout: number | null = null;

  constructor(private readonly baseUrl: string | null) {}

  subscribe(method: string, listener: SubscriptionListener) {
    if (!this.baseUrl) {
      throw new Error("Set VITE_TRADEZ_API_URL to enable backend subscriptions.");
    }
    let listeners = this.listeners.get(method);
    const isFirstListener = !listeners || listeners.size === 0;
    if (!listeners) {
      listeners = new Set();
      this.listeners.set(method, listeners);
    }
    listeners.add(listener);
    this.ensureSocket();
    if (isFirstListener && this.socket?.readyState === WebSocket.OPEN) {
      this.sendSubscriptionRequest(method);
    }
    return () => {
      this.unsubscribe(method, listener);
    };
  }

  private unsubscribe(method: string, listener: SubscriptionListener) {
    const listeners = this.listeners.get(method);
    if (!listeners) {
      return;
    }
    listeners.delete(listener);
    if (listeners.size === 0) {
      this.listeners.delete(method);
      this.subscribedMethods.delete(method);
      if (this.listeners.size === 0) {
        this.teardown();
      }
    }
  }

  private teardown() {
    if (this.reconnectTimeout) {
      if (typeof window !== "undefined") {
        window.clearTimeout(this.reconnectTimeout);
      }
      this.reconnectTimeout = null;
    }
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
    this.subscribedMethods.clear();
  }

  private ensureSocket() {
    if (!this.baseUrl) {
      return null;
    }
    if (
      this.socket &&
      (this.socket.readyState === WebSocket.OPEN ||
        this.socket.readyState === WebSocket.CONNECTING)
    ) {
      return this.socket;
    }
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
    if (typeof window === "undefined") {
      return null;
    }
    const socket = new WebSocket(this.baseUrl);
    this.socket = socket;

    socket.addEventListener("open", () => {
      this.subscribedMethods.clear();
      this.listeners.forEach((_listeners, method) => {
        this.sendSubscriptionRequest(method);
      });
    });

    socket.addEventListener("message", (event) => {
      try {
        const payload = JSON.parse(event.data);
        const method = payload?.method;
        if (!method) {
          return;
        }
        const listeners = this.listeners.get(method);
        if (!listeners) {
          return;
        }
        listeners.forEach((listener) => {
          try {
            listener(payload);
          } catch (error) {
            console.error(`Listener for ${method} threw:`, error);
          }
        });
      } catch (error) {
        console.error("Failed to parse websocket payload:", error);
      }
    });

    socket.addEventListener("error", (event) => {
      console.error("Websocket subscription error:", event);
    });

    socket.addEventListener("close", () => {
      this.socket = null;
      this.subscribedMethods.clear();
      if (this.listeners.size > 0 && !this.reconnectTimeout) {
        this.reconnectTimeout = window.setTimeout(() => {
          this.reconnectTimeout = null;
          this.ensureSocket();
        }, 1000);
      }
    });

    return socket;
  }

  private sendSubscriptionRequest(method: string) {
    if (this.subscribedMethods.has(method)) {
      return;
    }
    const socket = this.socket;
    if (!socket || socket.readyState !== WebSocket.OPEN) {
      return;
    }
    const requestId = `${method}-${Date.now()}-${Math.random()
      .toString(16)
      .slice(2)}`;
    socket.send(
      JSON.stringify({
        jsonrpc: "2.0",
        id: requestId,
        method,
        params: [],
      })
    );
    this.subscribedMethods.add(method);
  }
}

const subscriptionManager = new SubscriptionManager(WS_BASE_URL);

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
  const callRpc = useCallback(
    async <T>(method: string, params: unknown[]) => {
      if (!API_BASE_URL) {
        throw new Error("Set VITE_TRADEZ_API_URL to enable backend requests.");
      }
      const response = await fetch(API_BASE_URL, {
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
    [API_BASE_URL]
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

  const subscribeJsonRpc = useCallback((method: string, onMessage: (payload: any) => void) => {
    return subscriptionManager.subscribe(method, onMessage);
  }, []);

  const subscribeOrderbookState = useCallback(
    (onMessage: (state: RpcOrderbookState) => void) =>
      subscribeJsonRpc("subscribeOrderBookState", (payload) => {
        if (
          payload?.method === "subscribeOrderBookState" &&
          payload?.params?.result
        ) {
          onMessage(payload.params.result as RpcOrderbookState);
        }
      }),
    [subscribeJsonRpc]
  );

  const subscribeEvent = useCallback(
    (onMessage: (event: RpcEvent) => void) =>
      subscribeJsonRpc("subscribeEvent", (payload) => {
        if (payload?.method === "subscribeEvent" && payload?.params?.result) {
          onMessage(payload.params.result as RpcEvent);
        }
      }),
    [subscribeJsonRpc]
  );

  return {
    apiUrl: API_BASE_URL,
    isApiConfigured: Boolean(API_BASE_URL),
    sendOrder,
    cancelOrder,
    faucet,
    getBalances,
    getOrders,
    getOrderbookState,
    subscribeOrderbookState,
    subscribeEvent,
  };
};
