import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ethers } from "ethers";
import { Card } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { RpcEvent, useTradezApi } from "@/hooks/useTradezApi";
import { useMarket } from "@/contexts/MarketContext";

const DECIMALS = 6;

type OrderBookEntry = {
  price: string;
  size: string;
  total: string;
  totalRaw: bigint;
  priceRaw: bigint;
};

type HistoryEntry = {
  time: string;
  price: string;
  size: string;
  side: "Bid" | "Ask";
};

const formatDecimal = (value: bigint, fractionDigits: number) => {
  const formatted = ethers.formatUnits(value, DECIMALS);
  const numeric = Number.parseFloat(formatted);
  if (Number.isFinite(numeric)) {
    return numeric.toLocaleString(undefined, {
      minimumFractionDigits: fractionDigits,
      maximumFractionDigits: fractionDigits,
    });
  }
  return formatted;
};

const OrderBook = () => {
  const { subscribeEvent, subscribeOrderbookState, isApiConfigured } = useTradezApi();
  const { setBestBid: setGlobalBestBid } = useMarket();
  const [asks, setAsks] = useState<OrderBookEntry[]>([]);
  const [bids, setBids] = useState<OrderBookEntry[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [bestBid, setBestBid] = useState<string | null>(null);
  const [bestAsk, setBestAsk] = useState<string | null>(null);
  const [spread, setSpread] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const pollingRef = useRef<number | null>(null);
  const fetchingRef = useRef(false);

  const resetState = useCallback((message?: string) => {
    setAsks([]);
    setBids([]);
    setHistory([]);
    setBestAsk(null);
    setBestBid(null);
    setSpread(null);
    setError(message ?? null);
    setLoading(false);
  }, []);

  const mapLevels = useCallback((levels: Array<[number, number]>) => {
    let cumulative = 0n;
    const entries: OrderBookEntry[] = [];
    for (const [price, qty] of levels) {
      const qtyBig = BigInt(Math.trunc(qty));
      const priceBig = BigInt(Math.trunc(price));
      cumulative += qtyBig;
      entries.push({
        price: formatDecimal(priceBig, 4),
        size: formatDecimal(qtyBig, 3),
        total: formatDecimal(cumulative, 3),
        totalRaw: cumulative,
        priceRaw: priceBig,
      });
    }
    return entries;
  }, []);

  const eventToHistoryEntry = useCallback((event: RpcEvent): HistoryEntry | null => {
    if (!("Trade" in event)) {
      return null;
    }
    const { qty, price, origin_side } = event.Trade;
    const qtyBig = BigInt(Math.trunc(qty));
    const priceBig = BigInt(Math.trunc(price));
    return {
      time: new Date().toLocaleTimeString(undefined, { hour12: false }),
      price: formatDecimal(priceBig, 4),
      size: formatDecimal(qtyBig, 3),
      side: origin_side,
    };
  }, []);

  useEffect(() => {
    if (!isApiConfigured) {
      setHistory([]);
      return;
    }

    let unsubscribeHistory: (() => void) | null = null;
    try {
      unsubscribeHistory = subscribeEvent((event) => {
        setHistory((prev) => {
          const entry = eventToHistoryEntry(event);
          if (!entry) {
            return prev;
          }
          const next = [entry, ...prev];
          return next.slice(0, 200);
        });
      });
    } catch (err: any) {
      console.error("Failed to subscribe to trade history:", err);
    }

    return () => {
      if (unsubscribeHistory) {
        unsubscribeHistory();
      }
    };
  }, [eventToHistoryEntry, isApiConfigured, subscribeEvent]);

  useEffect(() => {
    if (!isApiConfigured) {
      resetState("Backend unavailable");
      return;
    }
    setLoading(true);
    let unsubscribe: (() => void) | null = null;
    try {
      unsubscribe = subscribeOrderbookState(([rawBids, rawAsks]) => {
        const mappedBids = mapLevels(rawBids);
        const mappedAsks = mapLevels(rawAsks);
        setBids(mappedBids);
        setAsks(mappedAsks);

        if (mappedBids.length > 0) {
          setBestBid(mappedBids[0].price);
          setGlobalBestBid(mappedBids[0].price);
        } else {
          setBestBid(null);
          setGlobalBestBid(null);
        }
        if (mappedAsks.length > 0) {
          setBestAsk(mappedAsks[0].price);
        } else {
          setBestAsk(null);
        }

        if (mappedBids.length > 0 && mappedAsks.length > 0) {
          const bidPrice = mappedBids[0].priceRaw;
          const askPrice = mappedAsks[0].priceRaw;
          const spreadRaw = askPrice > bidPrice ? askPrice - bidPrice : 0n;
          setSpread(formatDecimal(spreadRaw, 4));
        } else {
          setSpread(null);
        }
        setError(null);
        setLoading(false);
      });
    } catch (err: any) {
      console.error("Failed to subscribe to orderbook state:", err);
      setError(err?.message ?? "Unable to subscribe to orderbook state");
      setLoading(false);
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [isApiConfigured, mapLevels, resetState, setGlobalBestBid, subscribeOrderbookState]);

  const maxBidTotal = useMemo(
    () => bids.reduce((max, entry) => (entry.totalRaw > max ? entry.totalRaw : max), 0n),
    [bids]
  );
  const maxAskTotal = useMemo(
    () => asks.reduce((max, entry) => (entry.totalRaw > max ? entry.totalRaw : max), 0n),
    [asks]
  );

  const renderOrderBookSide = useCallback(
    (orders: OrderBookEntry[], isBid: boolean, maxTotal: bigint) => {
      if (!orders.length) {
        return (
          <div className="px-2 py-3 text-sm text-muted-foreground text-center border border-border/30 rounded">
            No orders
          </div>
        );
      }
      return (
        <div className="space-y-1 max-h-52 overflow-y-auto pr-1">
          {orders.map((order, idx) => {
            const ratio =
              maxTotal > 0n ? Number(order.totalRaw * 100n / maxTotal) : 0;
            const clampedRatio = Number.isFinite(ratio) ? Math.min(Math.max(ratio, 0), 100) : 0;
            return (
              <div
                key={`${order.price}-${idx}`}
                className="grid grid-cols-3 gap-2 text-sm py-1 px-2 hover:bg-secondary/30 rounded relative overflow-hidden"
              >
                <div
                  className={`absolute inset-y-0 right-0 ${
                    isBid ? "bg-buy/10" : "bg-sell/10"
                  }`}
                  style={{ width: `${clampedRatio}%` }}
                />
                <span className={`relative z-10 font-medium ${isBid ? "text-buy" : "text-sell"}`}>
                  {order.price}
                </span>
                <span className="relative z-10 text-right">{order.size}</span>
                <span className="relative z-10 text-right text-muted-foreground">
                  {order.total}
                </span>
              </div>
            );
          })}
        </div>
      );
    },
    []
  );

  const renderHistory = useCallback(() => {
    if (!history.length) {
      return (
        <div className="px-2 py-3 text-sm text-muted-foreground text-center border border-border/30 rounded">
          No trades yet
        </div>
      );
    }
    return (
      <div className="space-y-1 max-h-96 overflow-y-auto pr-1">
        {history.map((entry, idx) => (
          <div
            key={`${entry.time}-${idx}`}
            className="grid grid-cols-3 gap-2 text-sm py-1 px-2 rounded relative"
          >
            <span className="text-muted-foreground">{entry.time}</span>
            <span className={`text-right font-medium ${entry.side === "Bid" ? "text-buy" : "text-sell"}`}>
              {entry.price}
            </span>
            <span className="text-right">{entry.size}</span>
          </div>
        ))}
      </div>
    );
  }, [history]);

  return (
    <Card className="h-full bg-orderbook-bg border-border/50 p-4">
      <Tabs defaultValue="book" className="w-full">
        <TabsList className="w-full grid grid-cols-2 mb-4">
          <TabsTrigger value="book">Book</TabsTrigger>
          <TabsTrigger value="history">History</TabsTrigger>
        </TabsList>

        <TabsContent value="book" className="space-y-4 mt-0">
          <div className="grid grid-cols-3 gap-2 text-xs text-muted-foreground px-2">
            <span>Price (USDC)</span>
            <span className="text-right">Size (XTZ)</span>
            <span className="text-right">Total</span>
          </div>
          {error ? (
            <div className="px-2 py-3 text-sm text-red-500 border border-red-500/30 rounded bg-red-500/5">
              {error}
            </div>
          ) : (
            <>
              {renderOrderBookSide([...asks].reverse(), false, maxAskTotal)}

              <div className="py-3 px-2 bg-secondary/20 rounded text-center">
                <div className="text-xl font-bold text-buy">
                  {loading && !bestBid && !bestAsk
                    ? "Loading..."
                    : bestBid && bestAsk
                    ? `${bestBid} / ${bestAsk}`
                    : bestBid || bestAsk || "—"}
                </div>
                <div className="text-xs text-muted-foreground">
                  {spread ? `Spread: ${spread} USDC` : "Spread unavailable"}
                </div>
              </div>

              {renderOrderBookSide(bids, true, maxBidTotal)}
            </>
          )}
        </TabsContent>

        <TabsContent value="history" className="space-y-3 mt-0">
          <div className="grid grid-cols-3 gap-2 text-xs text-muted-foreground px-2">
            <span>Time</span>
            <span className="text-right">Price (USDC)</span>
            <span className="text-right">Size (XTZ)</span>
          </div>
          {error ? (
            <div className="px-2 py-3 text-sm text-red-500 border border-red-500/30 rounded bg-red-500/5">
              {error}
            </div>
          ) : (
            renderHistory()
          )}
        </TabsContent>
      </Tabs>
    </Card>
  );
};

export default OrderBook;
