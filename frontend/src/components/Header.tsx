import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Wallet, LogOut, Coins } from "lucide-react";
import { useWallet } from "@/hooks/useWallet";
import { useToast } from "@/hooks/use-toast";

const Header = () => {
  const { account, connecting, connectWallet, disconnectWallet } = useWallet();
  const { toast } = useToast();
  const [claiming, setClaiming] = useState(false);

  const handleClaimTestTokens = async () => {
    if (!account || claiming) {
      return;
    }
    const faucetUrl = import.meta.env.VITE_FAUCET_URL as string | undefined;
    if (!faucetUrl) {
      toast({
        title: "Faucet unavailable",
        description: "Set VITE_FAUCET_URL to enable test XTZ claims.",
        variant: "destructive",
      });
      return;
    }
    try {
      setClaiming(true);
      const response = await fetch(faucetUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ address: account }),
      });
      if (!response.ok) {
        const errorText = await response.text();
        throw new Error(errorText || "Unable to claim test tokens");
      }
      const result = (await response.json().catch(() => ({}))) as { amount?: number };
      toast({
        title: "Test XTZ claimed",
        description:
          result?.amount != null
            ? `Received ${result.amount} XTZ`
            : "Your faucet request was submitted.",
      });
    } catch (error: any) {
      console.error("Faucet claim failed:", error);
      toast({
        title: "Claim failed",
        description: error.message || "Unable to claim test tokens",
        variant: "destructive",
      });
    } finally {
      setClaiming(false);
    }
  };

  return (
    <header className="border-b border-border bg-card/50 backdrop-blur-sm sticky top-0 z-50">
      <div className="container mx-auto px-4 py-4 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-10 h-10 rounded-lg bg-primary/20 flex items-center justify-center">
            <span className="text-xl font-bold text-primary">DX</span>
          </div>
          <h1 className="text-2xl font-bold">
            Trad<span className="text-primary">EZ</span>
          </h1>
        </div>

        <div className="flex items-center gap-2">
          {account && (
            <Button
              variant="secondary"
              onClick={handleClaimTestTokens}
              disabled={claiming}
              className="flex items-center gap-2"
            >
              <Coins className="w-4 h-4" />
              {claiming ? "Claiming..." : "Claim test XTZ"}
            </Button>
          )}
          <Button
            onClick={connectWallet}
            disabled={connecting}
            className="bg-primary hover:bg-primary/90 text-primary-foreground font-semibold"
          >
            <Wallet className="w-4 h-4 mr-2" />
            {connecting
              ? "Connecting..."
              : account
              ? `${account.slice(0, 6)}...${account.slice(-4)}`
              : "Connect Wallet"}
          </Button>
          {account && (
            <Button
              variant="ghost"
              size="icon"
              onClick={disconnectWallet}
              aria-label="Disconnect wallet"
            >
              <LogOut className="w-4 h-4" />
            </Button>
          )}
        </div>
      </div>
    </header>
  );
};

export default Header;
