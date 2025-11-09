import { useState } from "react";
import { ethers } from "ethers";
import { Button } from "@/components/ui/button";
import { Wallet, LogOut, Coins } from "lucide-react";
import { useWallet } from "@/hooks/useWallet";
import { useToast } from "@/hooks/use-toast";
import { useTradezApi, RpcCurrency } from "@/hooks/useTradezApi";

const DEFAULT_FAUCET_AMOUNT = 1_000_000_000n;

const Header = () => {
  const { account, connecting, connectWallet, disconnectWallet, signMessage } = useWallet();
  const { toast } = useToast();
  const { faucet, isApiConfigured } = useTradezApi();
  const [claiming, setClaiming] = useState(false);
  const [claimingCurrency, setClaimingCurrency] = useState<RpcCurrency | null>(null);

  const requestFaucet = async (currency: RpcCurrency) => {
    if (!account || claimingCurrency) {
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
      setClaimingCurrency(currency);
      const amount = DEFAULT_FAUCET_AMOUNT;
      const encoded = ethers.encodeRlp([ethers.toBeArray(amount), ethers.toBeArray(currency === "XTZ" ? 1n : 0n)]);
      const messageBytes = ethers.getBytes(encoded);
      const signature = await signMessage(messageBytes);
      if (!signature) {
        throw new Error("Unable to sign faucet request");
      }
      const faucetPromise = faucet(
        {
          amount: Number(amount),
          currency,
        },
        signature
      );
      window.dispatchEvent(new CustomEvent("tradez:faucet-called"));
      const result = await faucetPromise;
      toast({
        title: `Test ${currency} claimed`,
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
      setClaimingCurrency(null);
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
            <>
              <Button
                variant="secondary"
                onClick={() => requestFaucet("XTZ")}
                disabled={claimingCurrency !== null}
                className="flex items-center gap-2"
              >
                <Coins className="w-4 h-4" />
                {claimingCurrency === "XTZ" ? "Claiming..." : "Claim test XTZ"}
              </Button>
              <Button
                variant="secondary"
                onClick={() => requestFaucet("USDC")}
                disabled={claimingCurrency !== null}
                className="flex items-center gap-2"
              >
                <Coins className="w-4 h-4" />
                {claimingCurrency === "USDC" ? "Claiming..." : "Claim test USDC"}
              </Button>
            </>
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
