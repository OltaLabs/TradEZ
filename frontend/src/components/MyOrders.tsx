import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ethers } from "ethers";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { RpcEvent, RpcOrdersResult, useTradezApi } from "@/hooks/useTradezApi";
import { useWallet } from "@/hooks/useWallet";
import { normalizeAddressLike } from "@/lib/address";

const DECIMALS = 6;
const PAIR_LABEL = "XTZ/USDC";

type DisplayOrder = {
  id: number;
  side: "buy" | "sell";
  type: string;
  price: string;
  amount: string;
  filled: string;
  remaining: string;
  nonce: number;
};

const formatDecimal = (value: number | bigint, fractionDigits: number) => {
  const big = typeof value === "bigint" ? value : BigInt(Math.trunc(value));
  const formatted = ethers.formatUnits(big, DECIMALS);
  const numeric = Number.parseFloat(formatted);
  if (Number.isFinite(numeric)) {
    return numeric.toLocaleString(undefined, {
      minimumFractionDigits: 0,
      maximumFractionDigits: fractionDigits,
    });
  }
  return formatted;
};

const mapOrders = (orders: RpcOrdersResult): DisplayOrder[] => {
  return orders.map(([orderId, order]) => {
    const qty = BigInt(Math.trunc(order.qty));
    const remaining = BigInt(Math.trunc(order.remaining));
    const filled = qty > remaining ? qty - remaining : 0n;

    return {
      id: orderId,
      side: order.side === "Bid" ? "buy" : "sell",
      type: order.ord_type.toLowerCase(),
      price: formatDecimal(order.price, 4),
      amount: formatDecimal(order.qty, 3),
      filled: formatDecimal(filled, 3),
      remaining: formatDecimal(order.remaining, 3),
      nonce: order.nonce,
    };
  });
};

const MyOrders = () => {
  const { account, signMessage } = useWallet();
  const { toast } = useToast();
  const { getOrders, cancelOrder, subscribeEvent, isApiConfigured } = useTradezApi();
  const [orders, setOrders] = useState<DisplayOrder[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [cancellingId, setCancellingId] = useState<number | null>(null);
  const fetchingRef = useRef(false);
  const orderIdsRef = useRef<Set<number>>(new Set());

  const resetState = useCallback((message?: string) => {
    setOrders([]);
    setLoading(false);
    setError(message ?? null);
  }, []);

  const fetchOrders = useCallback(
    async (silent = false) => {
      if (!account) {
        resetState(null);
        return;
      }
      if (!isApiConfigured) {
        resetState("Backend unavailable");
        return;
      }
      if (fetchingRef.current) {
        return;
      }
      fetchingRef.current = true;
      try {
        if (!silent) {
          setLoading(true);
        }
        const result = await getOrders(account);
        setOrders(mapOrders(result).sort((a, b) => b.nonce - a.nonce));
        setError(null);
      } catch (err: any) {
        console.error("Failed to fetch orders:", err);
        setError(err?.message ?? "Unable to load orders");
      } finally {
        fetchingRef.current = false;
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [account, getOrders, isApiConfigured, resetState]
  );

  useEffect(() => {
    orderIdsRef.current = new Set(orders.map((order) => order.id));
  }, [orders]);

  const notifyTradeEvent = useCallback(
    (event: RpcEvent, address: string) => {
      const normalized = address.toLowerCase();
      const showSuccessToast = (title: string, description: string) => {
        toast({
          title,
          description,
          duration: 500,
          className:
            "border border-emerald-500 bg-emerald-500 text-emerald-900 dark:text-emerald-100",
        });
      };

      if ("Trade" in event) {
        const makerUser = normalizeAddressLike(event.Trade.maker_user);
        const takerUser = normalizeAddressLike(event.Trade.taker_user);
        if (makerUser !== normalized && takerUser !== normalized) {
          return false;
        }
        const role = makerUser === normalized ? "Maker" : "Taker";
        const qtyLabel = formatDecimal(event.Trade.qty, 3);
        const priceLabel = formatDecimal(event.Trade.price, 4);
        showSuccessToast(
          "Order partially filled",
          `${role} fill: ${qtyLabel} XTZ @ $${priceLabel}`
        );
        return true;
      }

      if ("Done" in event) {
        const user = normalizeAddressLike(event.Done.user);
        if (user !== normalized) {
          return false;
        }
        showSuccessToast("Order completed", `Order #${event.Done.id} fully filled.`);
        return true;
      }

      return false;
    },
    [toast]
  );

  const eventTargetsAccount = useCallback(
    (event: RpcEvent, address: string) => {
      const normalizedAddress = address.toLowerCase();
      const knownIds = orderIdsRef.current;
      if ("Placed" in event) {
        const user = normalizeAddressLike(event.Placed.user);
        return user === normalizedAddress;
      }
      if ("Trade" in event) {
        const makerUser = normalizeAddressLike(event.Trade.maker_user);
        const takerUser = normalizeAddressLike(event.Trade.taker_user);
        return (
          makerUser === normalizedAddress ||
          takerUser === normalizedAddress ||
          knownIds.has(event.Trade.maker_id) ||
          knownIds.has(event.Trade.taker_id)
        );
      }
      if ("Done" in event) {
        const user = normalizeAddressLike(event.Done.user);
        return user === normalizedAddress || knownIds.has(event.Done.id);
      }
      if ("Cancelled" in event) {
        const user = normalizeAddressLike(event.Cancelled.user);
        return user === normalizedAddress || knownIds.has(event.Cancelled.id);
      }
      return false;
    },
    []
  );

  useEffect(() => {
    if (!account || !isApiConfigured) {
      resetState(account ? "Backend unavailable" : null);
      return;
    }

    fetchOrders().catch(() => undefined);

    let unsubscribe: (() => void) | null = null;
    try {
      unsubscribe = subscribeEvent((event) => {
        const affectsAccount = eventTargetsAccount(event, account);
        notifyTradeEvent(event, account);
        if (affectsAccount) {
          fetchOrders(true).catch(() => undefined);
        }
      });
    } catch (err: any) {
      console.error("Failed to subscribe to events:", err);
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [account, eventTargetsAccount, fetchOrders, isApiConfigured, notifyTradeEvent, resetState, subscribeEvent]);

  const handleCancelOrder = useCallback(
    async (orderId: number) => {
      if (!account) {
        toast({
          title: "Wallet not connected",
          description: "Connect your wallet to cancel orders.",
          variant: "destructive",
        });
        return;
      }
      if (!isApiConfigured) {
        toast({
          title: "Backend unavailable",
          description: "Configure VITE_TRADEZ_API_URL to cancel orders.",
          variant: "destructive",
        });
        return;
      }
      try {
        setCancellingId(orderId);
        const encoded = ethers.encodeRlp([ethers.toBeArray(BigInt(orderId))]);
        const signature = await signMessage(ethers.getBytes(encoded));
        if (!signature) {
          throw new Error("Unable to sign cancel request");
        }
        await cancelOrder({ order_id: orderId }, signature);
        toast({
          title: "Order cancelled",
          description: `Order ${orderId} cancellation submitted.`,
        });
        fetchOrders(true).catch(() => undefined);
      } catch (err: any) {
        console.error("Failed to cancel order:", err);
        toast({
          title: "Cancel failed",
          description: err?.message ?? "Unable to cancel order.",
          variant: "destructive",
        });
      } finally {
        setCancellingId((prev) => (prev === orderId ? null : prev));
      }
    },
    [account, cancelOrder, fetchOrders, isApiConfigured, signMessage, toast]
  );

  const content = useMemo(() => {
    if (!account) {
      return (
        <div className="text-center py-8 text-muted-foreground">
          Connect your wallet to view active orders.
        </div>
      );
    }
    if (error) {
      return (
        <div className="text-center py-8 text-red-500 border border-red-500/30 rounded bg-red-500/5">
          {error}
        </div>
      );
    }
    if (loading && orders.length === 0) {
      return <div className="text-center py-8 text-muted-foreground">Loading orders…</div>;
    }
    if (orders.length === 0) {
      return (
        <div className="text-center py-8 text-muted-foreground">
          <p>No active orders</p>
        </div>
      );
    }

    return (
      <div className="space-y-2">
        {orders.map((order) => (
          <div
            key={order.id}
            className="p-3 bg-secondary/30 rounded border border-border/30 hover:border-border/60 transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-1">
                  <span className="font-semibold">{PAIR_LABEL}</span>
                  <span
                    className={`text-xs px-2 py-0.5 rounded ${
                      order.side === "buy" ? "bg-buy/20 text-buy" : "bg-sell/20 text-sell"
                    }`}
                  >
                    {order.side.toUpperCase()}
                  </span>
                  <span className="text-xs text-muted-foreground">{order.type}</span>
                  <span className="text-xs text-muted-foreground">ID: {order.id}</span>
                </div>

                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div>
                    <span className="text-muted-foreground">Price:</span>{" "}
                    <span className="font-medium">${order.price}</span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Amount:</span>{" "}
                    <span className="font-medium">{order.amount} XTZ</span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Filled:</span>{" "}
                    <span className="font-medium">
                      {order.filled} / {order.amount}
                    </span>
                  </div>
                  <div>
                    <span className="text-muted-foreground">Remaining:</span>{" "}
                    <span className="font-medium">{order.remaining} XTZ</span>
                  </div>
                </div>
              </div>

              <Button
                variant="ghost"
                size="icon"
                onClick={() => handleCancelOrder(order.id)}
                disabled={cancellingId === order.id}
                className="ml-2 hover:bg-destructive/20 hover:text-destructive"
              >
                <X className="w-4 h-4" />
              </Button>
            </div>
          </div>
        ))}
      </div>
    );
  }, [account, cancellingId, error, handleCancelOrder, loading, orders]);

  return (
    <Card className="bg-card border-border/50 p-4">
      <h3 className="text-lg font-semibold mb-4">My Orders</h3>
      {content}
    </Card>
  );
};

export default MyOrders;
