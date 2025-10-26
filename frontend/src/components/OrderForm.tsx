import { useState } from "react";
import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useWallet } from "@/hooks/useWallet";
import { useToast } from "@/hooks/use-toast";
import { useBalances } from "@/hooks/useBalances";

const OrderForm = () => {
  const { account, signMessage } = useWallet();
  const { toast } = useToast();
  const [orderType, setOrderType] = useState<"limit" | "market">("limit");
  const [price, setPrice] = useState("1.2353");
  const [amount, setAmount] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const {
    xtz: xtzBalance,
    usdc: usdcBalance,
    loading: balancesLoading,
  } = useBalances(account);

  const handlePlaceOrder = async (side: "buy" | "sell") => {
    if (!account) {
      toast({
        title: "Wallet not connected",
        description: "Please connect your wallet to place orders",
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

      // Create order message to sign
      const orderMessage = {
        pair: "XTZ/USDC",
        side,
        type: orderType,
        price: orderType === "limit" ? price : "market",
        amount,
        timestamp: Date.now(),
      };

      const messageToSign = JSON.stringify(orderMessage, null, 2);

      // Sign the message
      const signature = await signMessage(messageToSign);

      if (!signature) {
        throw new Error("Failed to sign message");
      }

      // Send to API (mock for now)
      console.log("Sending order to API:", {
        order: orderMessage,
        signature,
        account,
      });

      // Mock API call
      await new Promise((resolve) => setTimeout(resolve, 1000));

      toast({
        title: "Order placed successfully",
        description: `${side.toUpperCase()} ${amount} XTZ at ${
          orderType === "limit" ? `$${price}` : "market price"
        }`,
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
              disabled={!account || submitting}
              className="bg-buy hover:bg-buy-hover text-buy-foreground font-semibold"
            >
              {submitting ? "Placing..." : "Buy XTZ"}
            </Button>
            <Button
              onClick={() => handlePlaceOrder("sell")}
              disabled={!account || submitting}
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
              disabled={!account || submitting}
              className="bg-buy hover:bg-buy-hover text-buy-foreground font-semibold"
            >
              {submitting ? "Placing..." : "Buy XTZ"}
            </Button>
            <Button
              onClick={() => handlePlaceOrder("sell")}
              disabled={!account || submitting}
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
