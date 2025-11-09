import { createContext, useContext, useMemo, useState, ReactNode } from "react";

type MarketContextValue = {
  bestBid: string | null;
  setBestBid: (value: string | null) => void;
};

const MarketContext = createContext<MarketContextValue | undefined>(undefined);

export const MarketProvider = ({ children }: { children: ReactNode }) => {
  const [bestBid, setBestBid] = useState<string | null>(null);
  const value = useMemo(() => ({ bestBid, setBestBid }), [bestBid]);
  return <MarketContext.Provider value={value}>{children}</MarketContext.Provider>;
};

export const useMarket = () => {
  const context = useContext(MarketContext);
  if (!context) {
    throw new Error("useMarket must be used within MarketProvider");
  }
  return context;
};
