import { useEffect, useRef } from "react";
import { Card } from "@/components/ui/card";
import { TrendingUp, TrendingDown } from "lucide-react";

declare global {
  interface Window {
    TradingView?: any;
  }
}

const loadTradingViewScript = (() => {
  let loadPromise: Promise<void> | null = null;
  return () => {
    if (loadPromise) {
      return loadPromise;
    }
    loadPromise = new Promise<void>((resolve, reject) => {
      const existingScript = document.querySelector<HTMLScriptElement>('script[src="https://s3.tradingview.com/tv.js"]');
      if (existingScript && window.TradingView) {
        resolve();
        return;
      }
      const script = document.createElement("script");
      script.src = "https://s3.tradingview.com/tv.js";
      script.async = true;
      script.onload = () => resolve();
      script.onerror = () => reject(new Error("Failed to load TradingView script"));
      document.head.appendChild(script);
    });
    return loadPromise;
  };
})();

const TradingChart = () => {
  // Mock data - replace with live metrics if available
  const currentPrice = 1.2345;
  const priceChange = 2.34;
  const isPositive = priceChange > 0;
  const widgetContainerRef = useRef<HTMLDivElement | null>(null);
  const widgetContainerIdRef = useRef(`tv-chart-${Math.random().toString(36).slice(2)}`);

  useEffect(() => {
    let isMounted = true;

    const initWidget = async () => {
      try {
        await loadTradingViewScript();
        if (!isMounted || !window.TradingView || !widgetContainerRef.current) {
          return;
        }
        widgetContainerRef.current.innerHTML = "";
        /* eslint-disable camelcase */
        new window.TradingView.widget({
          autosize: true,
          symbol: "BINANCE:XTZUSDC",
          interval: "60",
          timezone: "Etc/UTC",
          theme: "dark",
          style: "1",
          locale: "en",
          toolbar_bg: "rgba(0, 0, 0, 0)",
          enable_publishing: false,
          hide_top_toolbar: false,
          hide_legend: false,
          save_image: false,
          container_id: widgetContainerIdRef.current,
        });
        /* eslint-enable camelcase */
      } catch (error) {
        console.error(error);
      }
    };

    initWidget();

    return () => {
      isMounted = false;
      if (widgetContainerRef.current) {
        widgetContainerRef.current.innerHTML = "";
      }
    };
  }, []);

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

      <div
        ref={widgetContainerRef}
        id={widgetContainerIdRef.current}
        className="w-full h-[400px] rounded-lg border border-border/30"
      />

      {/* Mock stats */}
      <div className="grid grid-cols-4 gap-4 mt-4">
        <div>
          <p className="text-xs text-muted-foreground">24h High</p>
          <p className="text-sm font-semibold">TBD</p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">24h Low</p>
          <p className="text-sm font-semibold">TBD</p>
        </div>
        <div>
          <p className="text-xs text-muted-foreground">24h Volume</p>
          <p className="text-sm font-semibold">TBD</p>
        </div>
      </div>
    </Card>
  );
};

export default TradingChart;
