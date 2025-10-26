import { Card } from "@/components/ui/card";
import { TrendingUp, TrendingDown } from "lucide-react";

const TradingChart = () => {
  // Mock data - in a real app, this would come from an API
  const currentPrice = 1.2345;
  const priceChange = 2.34;
  const isPositive = priceChange > 0;

  return (
    <Card className="h-full bg-chart-bg border-border/50 p-4">
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-2xl font-bold flex items-center gap-2">
            XTZ/USDC
            {isPositive ? (
              <TrendingUp className="w-5 h-5 text-buy" />
            ) : (
              <TrendingDown className="w-5 h-5 text-sell" />
            )}
          </h2>
          <div className="flex items-center gap-3 mt-1">
            <span className="text-xl font-semibold">${currentPrice}</span>
            <span
              className={`text-sm font-medium ${
                isPositive ? "text-buy" : "text-sell"
              }`}
            >
              {isPositive ? "+" : ""}
              {priceChange}%
            </span>
          </div>
        </div>
      </div>

      {/* Placeholder for chart */}
      <div className="w-full h-[400px] bg-background/50 rounded-lg flex items-center justify-center border border-border/30">
        <div className="text-center text-muted-foreground">
          <div className="mb-2">📊</div>
          <p>Chart will be displayed here</p>
          <p className="text-xs mt-1">Real-time price data integration pending</p>
        </div>
      </div>

      {/* Mock stats */}
      <div className="grid grid-cols-4 gap-4 mt-4">
        <div>
          <p className="text-xs text-muted-foreground">24h High</p>
          <p className="text-sm font-semibold">$1.2567</p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">24h Low</p>
          <p className="text-sm font-semibold">$1.2103</p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">24h Volume</p>
          <p className="text-sm font-semibold">$2.4M</p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">Spread</p>
          <p className="text-sm font-semibold">0.01%</p>
        </div>
      </div>
    </Card>
  );
};

export default TradingChart;
