import { Card } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface OrderBookEntry {
  price: number;
  size: number;
  total: number;
}

const OrderBook = () => {
  // Mock order book data
  const asks: OrderBookEntry[] = [
    { price: 1.2358, size: 125.5, total: 155.1 },
    { price: 1.2357, size: 89.2, total: 110.3 },
    { price: 1.2356, size: 203.1, total: 251.0 },
    { price: 1.2355, size: 156.7, total: 193.6 },
    { price: 1.2354, size: 94.3, total: 116.5 },
  ];

  const bids: OrderBookEntry[] = [
    { price: 1.2353, size: 178.4, total: 220.3 },
    { price: 1.2352, size: 112.8, total: 139.3 },
    { price: 1.2351, size: 245.6, total: 303.2 },
    { price: 1.2350, size: 67.9, total: 83.9 },
    { price: 1.2349, size: 198.2, total: 244.8 },
  ];

  const renderOrderBookSide = (orders: OrderBookEntry[], isBid: boolean) => (
    <div className="space-y-1">
      {orders.map((order, idx) => (
        <div
          key={idx}
          className="grid grid-cols-3 gap-2 text-sm py-1 px-2 hover:bg-secondary/30 rounded relative overflow-hidden"
        >
          <div
            className={`absolute inset-y-0 right-0 ${
              isBid ? "bg-buy/10" : "bg-sell/10"
            }`}
            style={{ width: `${(order.total / 300) * 100}%` }}
          />
          <span className={`relative z-10 font-medium ${isBid ? "text-buy" : "text-sell"}`}>
            {order.price.toFixed(4)}
          </span>
          <span className="relative z-10 text-right">{order.size.toFixed(1)}</span>
          <span className="relative z-10 text-right text-muted-foreground">
            {order.total.toFixed(1)}
          </span>
        </div>
      ))}
    </div>
  );

  return (
    <Card className="h-full bg-orderbook-bg border-border/50 p-4">
      <Tabs defaultValue="both" className="w-full">
        <TabsList className="w-full grid grid-cols-3 mb-4">
          <TabsTrigger value="both">Book</TabsTrigger>
          <TabsTrigger value="bids">Bids</TabsTrigger>
          <TabsTrigger value="asks">Asks</TabsTrigger>
        </TabsList>

        <div className="grid grid-cols-3 gap-2 text-xs text-muted-foreground mb-2 px-2">
          <span>Price (USDC)</span>
          <span className="text-right">Size (XTZ)</span>
          <span className="text-right">Total</span>
        </div>

        <TabsContent value="both" className="space-y-4 mt-0">
          {renderOrderBookSide(asks.slice().reverse(), false)}
          
          <div className="py-3 px-2 bg-secondary/20 rounded text-center">
            <div className="text-xl font-bold text-buy">1.2353</div>
            <div className="text-xs text-muted-foreground">Spread: 0.0005 (0.04%)</div>
          </div>
          
          {renderOrderBookSide(bids, true)}
        </TabsContent>

        <TabsContent value="bids" className="mt-0">
          {renderOrderBookSide(bids, true)}
        </TabsContent>

        <TabsContent value="asks" className="mt-0">
          {renderOrderBookSide(asks, false)}
        </TabsContent>
      </Tabs>
    </Card>
  );
};

export default OrderBook;
