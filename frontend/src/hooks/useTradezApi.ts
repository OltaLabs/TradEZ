import { useCallback, useMemo } from "react";

type RequestOptions = Omit<RequestInit, "body"> & {
  json?: unknown;
};

export type BalanceRequest = {
  ticker: string;
  address: string;
};

export type BalanceResponse = {
  ticker: string;
  balance: string;
};

export type FaucetRequest = {
  address: string;
  ticker?: string;
  amount?: string;
};

export type FaucetResponse = {
  ticker: string;
  amount: string;
  txHash?: string;
  message?: string;
};

export type PlaceOrderPayload = {
  pair: string;
  side: "buy" | "sell";
  type: "limit" | "market";
  price?: string;
  amount: string;
  signature: string;
  timestamp: number;
};

export type PlaceOrderResponse = {
  orderId: string;
  status: "accepted" | "pending" | "rejected";
  message?: string;
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

export const useTradezApi = () => {
  const apiBaseUrl = useMemo(
    () => trimTrailingSlash(import.meta.env.VITE_TRADEZ_API_URL as string | undefined),
    []
  );
  const faucetUrl = useMemo(() => {
    const explicit = trimTrailingSlash(import.meta.env.VITE_FAUCET_URL as string | undefined);
    if (explicit) {
      return explicit;
    }
    return apiBaseUrl ? `${apiBaseUrl}/faucet` : undefined;
  }, [apiBaseUrl]);

  const requestJson = useCallback(async <T>(url: string, options: RequestOptions = {}) => {
    const { json, ...init } = options;
    const headers: HeadersInit = {
      Accept: "application/json",
      ...(json ? { "Content-Type": "application/json" } : {}),
      ...(init.headers ?? {}),
    };

    const response = await fetch(url, {
      ...init,
      headers,
      body: json ? JSON.stringify(json) : init.body,
    });

    const payload = await parseBody(response);
    if (!response.ok) {
      const message =
        (typeof payload === "string" && payload) ||
        (payload && typeof payload === "object" && "message" in payload
          ? String((payload as Record<string, unknown>).message)
          : null) ||
        `Request failed with status ${response.status}`;
      throw new Error(message);
    }

    return payload as T;
  }, []);

  const callApi = useCallback(
    async <T>(path: string, options: RequestOptions = {}) => {
      if (!apiBaseUrl) {
        throw new Error("Set VITE_TRADEZ_API_URL to enable backend requests.");
      }
      const normalizedPath = path.startsWith("/") ? path : `/${path}`;
      const url = `${apiBaseUrl}${normalizedPath}`;
      return requestJson<T>(url, options);
    },
    [apiBaseUrl, requestJson]
  );

  const getBalance = useCallback(
    async ({ ticker, address }: BalanceRequest) => {
      const query = new URLSearchParams({ address });
      return callApi<BalanceResponse>(`/balances/${encodeURIComponent(ticker)}?${query.toString()}`);
    },
    [callApi]
  );

  const requestTestTokens = useCallback(
    async (payload: FaucetRequest) => {
      if (!faucetUrl) {
        throw new Error("Set VITE_FAUCET_URL or VITE_TRADEZ_API_URL to enable faucet requests.");
      }
      return requestJson<FaucetResponse>(faucetUrl, {
        method: "POST",
        json: payload,
      });
    },
    [faucetUrl, requestJson]
  );

  const placeOrder = useCallback(
    async (payload: PlaceOrderPayload) => {
      return callApi<PlaceOrderResponse>("/orders", {
        method: "POST",
        json: payload,
      });
    },
    [callApi]
  );

  return {
    apiUrl: apiBaseUrl,
    faucetUrl,
    isApiConfigured: Boolean(apiBaseUrl),
    getBalance,
    requestTestTokens,
    placeOrder,
  };
};
