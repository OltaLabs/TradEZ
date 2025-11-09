import { useCallback, useEffect, useRef, useState } from "react";
import { ethers } from "ethers";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useWallet } from "@/hooks/useWallet";
import { useToast } from "@/hooks/use-toast";
import { RpcEvent, useTradezApi } from "@/hooks/useTradezApi";
import { normalizeAddressLike } from "@/lib/address";

const DECIMALS = 6;

const OrderForm = () => {
  const { account, signMessage } = useWallet();
  const { sendOrder, getBalances, subscribeEvent, isApiConfigured } = useTradezApi();
  const { toast } = useToast();
  const [orderType, setOrderType] = useState<"limit" | "market">("limit");
  const [price, setPrice] = useState("1.2353");
  const [amount, setAmount] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [balancesLoading, setBalancesLoading] = useState(false);
  const [balanceError, setBalanceError] = useState<string | null>(null);
  const [xtzBalance, setXtzBalance] = useState<string | null>(null);
  const [usdcBalance, setUsdcBalance] = useState<string | null>(null);
  const faucetTimeoutRef = useRef<number | null>(null);

  const resetBalances = useCallback(() => {
    setXtzBalance(null);
    setUsdcBalance(null);
    setBalancesLoading(false);
    setBalanceError(null);
  }, []);

  const fetchBalances = useCallback(
    async (silent = false) => {
      if (!account || !isApiConfigured) {
        resetBalances();
        return;
      }
      try {
        if (!silent) {
          setBalancesLoading(true);
        }
        const result = await getBalances(account);
        let nextXtz: string | null = "0";
        let nextUsdc: string | null = "0";
        for (const [currency, value] of result) {
          const units = BigInt(Math.trunc(value));
          const formatted = ethers.formatUnits(units, DECIMALS);
          if (currency === "XTZ") {
            nextXtz = formatted;
          } else if (currency === "USDC") {
            nextUsdc = formatted;
          }
        }
        setXtzBalance(nextXtz);
        setUsdcBalance(nextUsdc);
        setBalanceError(null);
      } catch (error: any) {
        console.error("Failed to fetch balances:", error);
        setBalanceError(error?.message ?? "Unable to load balances");
      } finally {
        if (!silent) {
          setBalancesLoading(false);
        }
      }
    },
    [account, getBalances, isApiConfigured, resetBalances]
  );

  const eventMentionsAccount = useCallback(
    (event: RpcEvent, address: string) => {
      const normalized = address.toLowerCase();
      if ("Placed" in event) {
        const user = normalizeAddressLike(event.Placed.user);
        return user === normalized;
      }
      if ("Trade" in event) {
        const makerUser = normalizeAddressLike(event.Trade.maker_user);
        const takerUser = normalizeAddressLike(event.Trade.taker_user);
        return (
          makerUser === normalized ||
          takerUser === normalized
        );
      }
      if ("Cancelled" in event) {
        const user = normalizeAddressLike(event.Cancelled.user);
        return user === normalized;
      }
      return false;
    },
    []
  );

  useEffect(() => {
    if (!account || !isApiConfigured) {
      resetBalances();
      return;
    }

    fetchBalances().catch(() => undefined);

    let unsubscribe: (() => void) | null = null;
    try {
      unsubscribe = subscribeEvent((event) => {
        if (eventMentionsAccount(event, account)) {
          fetchBalances(true).catch(() => undefined);
        }
      });
    } catch (err) {
      console.error("Failed to subscribe for balance updates:", err);
    }

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [account, eventMentionsAccount, fetchBalances, isApiConfigured, resetBalances, subscribeEvent]);

  useEffect(() => {
    const handleFaucetCall = () => {
      if (!account || !isApiConfigured) {
        return;
      }
      if (faucetTimeoutRef.current) {
        clearTimeout(faucetTimeoutRef.current);
      }
      faucetTimeoutRef.current = window.setTimeout(() => {
        faucetTimeoutRef.current = null;
        fetchBalances(true).catch(() => undefined);
      }, 1000);
    };

    window.addEventListener("tradez:faucet-called", handleFaucetCall);
    return () => {
      window.removeEventListener("tradez:faucet-called", handleFaucetCall);
      if (faucetTimeoutRef.current) {
        clearTimeout(faucetTimeoutRef.current);
        faucetTimeoutRef.current = null;
      }
    };
  }, [account, fetchBalances, isApiConfigured]);

  const handlePlaceOrder = async (side: "buy" | "sell") => {
    if (!account) {
      toast({
        title: "Wallet not connected",
        description: "Please connect your wallet to place orders",
        variant: "destructive",
      });
      return;
    }

    if (!isApiConfigured) {
      toast({
        title: "Backend unavailable",
        description: "Configure VITE_TRADEZ_API_URL to place orders.",
        variant: "destructive",
      });
      return;
    }

    if (!amount || parseFloat(amount) <= 0) {
      toast({
        title: "Invalid amount",
        description: "Please enter a valid amount",
        variant: "destructive",
      });
      return;
    }

    try {
      setSubmitting(true);

      const sizeUnits = ethers.parseUnits(amount, DECIMALS);
      const priceUnits = orderType === "limit" ? ethers.parseUnits(price, DECIMALS) : 0n;
      const nonce = BigInt(Date.now());
      const apiOrder = {
        side: side === "buy" ? ("Bid" as const) : ("Ask" as const),
        size: Number(sizeUnits),
        price: Number(priceUnits),
        nonce: Number(nonce),
      };
      const toMinimalBytes = (value: bigint) => ethers.toBeArray(value);
      const orderForSignature = [
        toMinimalBytes(side === "buy" ? 0n : 1n),
        toMinimalBytes(sizeUnits),
        toMinimalBytes(priceUnits),
        toMinimalBytes(nonce),
      ];
      const encodedOrder = ethers.encodeRlp(orderForSignature);
      const signature = await signMessage(ethers.getBytes(encodedOrder));
      if (!signature) {
        throw new Error("Failed to sign message");
      }

      const response = await sendOrder(apiOrder, signature);
      toast({
        title: "Order placed successfully",
        description: response ?? `${side.toUpperCase()} order submitted.`,
      });

      // Reset form
      setAmount("");
    } catch (error: any) {
      console.error("Error placing order:", error);
      toast({
        title: "Failed to place order",
        description: error.message || "An error occurred",
        variant: "destructive",
      });
    } finally {
      setSubmitting(false);
    }
  };

  const total = amount && price ? (parseFloat(amount) * parseFloat(price)).toFixed(2) : "0.00";

  const getBalanceDisplay = (value: string | null, fallback?: string) => {
    if (!account) {
      return "Connect wallet";
    }
    if (!isApiConfigured) {
      return "Backend unavailable";
    }
    if (balanceError) {
      return balanceError;
    }
    if (balancesLoading && !value) {
      return "Loading...";
    }
    if (!value) {
      return fallback ?? "0";
    }
    const parsed = Number(value);
    if (!Number.isFinite(parsed)) {
      return value;
    }
    return parsed.toLocaleString(undefined, { maximumFractionDigits: 4 });
  };

  const BalanceSummary = () => (
    <div className="mt-3 space-y-1 rounded border border-border/40 bg-secondary/10 p-3 text-xs">
      <div className="flex items-center justify-between">
        <span className="text-muted-foreground">Balance (XTZ)</span>
        <span className="font-semibold">
          {getBalanceDisplay(xtzBalance)}
        </span>
      </div>
      <div className="flex items-center justify-between">
        <span className="text-muted-foreground">Balance (USDC)</span>
        <span className="font-semibold">
          {getBalanceDisplay(usdcBalance)}
        </span>
      </div>
      <div className="flex items-center justify-between">
        <span className="text-muted-foreground">Fees</span>
        <span className="font-semibold text-muted-foreground">TODO</span>
      </div>
    </div>
  );

  return (
    <Card className="bg-card border-border/50 p-4">
      <Tabs defaultValue="limit" onValueChange={(v) => setOrderType(v as "limit" | "market")}>
        <TabsList className="w-full grid grid-cols-2 mb-4">
          <TabsTrigger value="limit">Limit</TabsTrigger>
          <TabsTrigger value="market">Market</TabsTrigger>
        </TabsList>

        <TabsContent value="limit" className="mt-0 space-y-4">
          <div>
            <Label htmlFor="limit-price" className="text-xs text-muted-foreground">
              Price (USDC)
            </Label>
            <Input
              id="limit-price"
              type="number"
              step="0.0001"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              className="mt-1"
            />
          </div>

          <div>
            <Label htmlFor="limit-amount" className="text-xs text-muted-foreground">
              Amount (XTZ)
            </Label>
            <Input
              id="limit-amount"
              type="number"
              step="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="mt-1"
            />
          </div>

          <div className="p-3 bg-secondary/30 rounded">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Total</span>
              <span className="font-semibold">${total} USDC</span>
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <Button
              onClick={() => handlePlaceOrder("buy")}
              disabled={!account || submitting || !isApiConfigured}
              className="bg-buy hover:bg-buy-hover text-buy-foreground font-semibold"
            >
              {submitting ? "Placing..." : "Buy XTZ"}
            </Button>
            <Button
              onClick={() => handlePlaceOrder("sell")}
              disabled={!account || submitting || !isApiConfigured}
              className="bg-sell hover:bg-sell-hover text-sell-foreground font-semibold"
            >
              {submitting ? "Placing..." : "Sell XTZ"}
            </Button>
          </div>

          <BalanceSummary />
        </TabsContent>

        <TabsContent value="market" className="mt-0 space-y-4">
          <div>
            <Label htmlFor="market-amount" className="text-xs text-muted-foreground">
              Amount (XTZ)
            </Label>
            <Input
              id="market-amount"
              type="number"
              step="0.01"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="0.00"
              className="mt-1"
            />
          </div>

          <div className="p-3 bg-secondary/30 rounded">
            <div className="text-sm text-center text-muted-foreground">
              Market orders execute at the best available price
            </div>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <Button
              onClick={() => handlePlaceOrder("buy")}
              disabled={!account || submitting || !isApiConfigured}
              className="bg-buy hover:bg-buy-hover text-buy-foreground font-semibold"
            >
              {submitting ? "Placing..." : "Buy XTZ"}
            </Button>
            <Button
              onClick={() => handlePlaceOrder("sell")}
              disabled={!account || submitting || !isApiConfigured}
              className="bg-sell hover:bg-sell-hover text-sell-foreground font-semibold"
            >
              {submitting ? "Placing..." : "Sell XTZ"}
            </Button>
          </div>

          <BalanceSummary />
        </TabsContent>
      </Tabs>

      {!account && (
        <div className="mt-4 p-3 bg-primary/10 border border-primary/20 rounded text-sm text-center">
          Connect your wallet to start trading
        </div>
      )}
    </Card>
  );
};

export default OrderForm;
