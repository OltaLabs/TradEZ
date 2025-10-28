import { useState } from "react";
import { ethers } from "ethers";
import { Button } from "@/components/ui/button";
import { Wallet, LogOut, Coins } from "lucide-react";
import { useWallet } from "@/hooks/useWallet";
import { useToast } from "@/hooks/use-toast";
import { useTradezApi } from "@/hooks/useTradezApi";

const DEFAULT_FAUCET_AMOUNT = 1_000_000_000n;

const Header = () => {
  const { account, connecting, connectWallet, disconnectWallet, signMessage } = useWallet();
  const { toast } = useToast();
  const { faucet, isApiConfigured } = useTradezApi();
  const [claiming, setClaiming] = useState(false);

  const handleClaimTestTokens = async () => {
    if (!account || claiming) {
      return;
    }
    if (!isApiConfigured) {
      toast({
        title: "Faucet unavailable",
        description: "Configure VITE_TRADEZ_API_URL to enable faucet requests.",
        variant: "destructive",
      });
      return;
    }
    try {
      setClaiming(true);
      const amount = DEFAULT_FAUCET_AMOUNT;
      const encoded = ethers.encodeRlp([ethers.toBeArray(amount)]);
      const messageBytes = ethers.getBytes(encoded);
      const signature = await signMessage(messageBytes);
      if (!signature) {
        throw new Error("Unable to sign faucet request");
      }
      const result = await faucet(
        {
          amount: Number(amount),
        },
        signature
      );
      toast({
        title: "Test XTZ claimed",
        description: result || "Your faucet request was submitted.",
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
