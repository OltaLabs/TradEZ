import { Card } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";
import { useToast } from "@/hooks/use-toast";

interface Order {
  id: string;
  pair: string;
  side: "buy" | "sell";
  type: string;
  price: string;
  amount: string;
  filled: string;
  timestamp: number;
}

const MyOrders = () => {
  const { toast } = useToast();

  // Mock orders - in a real app, this would come from state management
  const orders: Order[] = [
    {
      id: "1",
      pair: "XTZ/USDC",
      side: "buy",
      type: "limit",
      price: "1.2340",
      amount: "100.00",
      filled: "0.00",
      timestamp: Date.now() - 300000,
    },
    {
      id: "2",
      pair: "XTZ/USDC",
      side: "sell",
      type: "limit",
      price: "1.2370",
      amount: "75.50",
      filled: "25.00",
      timestamp: Date.now() - 600000,
    },
  ];

  const handleCancelOrder = (orderId: string) => {
    toast({
      title: "Order cancelled",
      description: `Order ${orderId} has been cancelled`,
    });
    console.log("Cancelling order:", orderId);
  };

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <Card className="bg-card border-border/50 p-4">
      <h3 className="text-lg font-semibold mb-4">My Orders</h3>

      {orders.length === 0 ? (
        <div className="text-center py-8 text-muted-foreground">
          <p>No active orders</p>
        </div>
      ) : (
        <div className="space-y-2">
          {orders.map((order) => (
            <div
              key={order.id}
              className="p-3 bg-secondary/30 rounded border border-border/30 hover:border-border/60 transition-colors"
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-semibold">{order.pair}</span>
                    <span
                      className={`text-xs px-2 py-0.5 rounded ${
                        order.side === "buy"
                          ? "bg-buy/20 text-buy"
                          : "bg-sell/20 text-sell"
                      }`}
                    >
                      {order.side.toUpperCase()}
                    </span>
                    <span className="text-xs text-muted-foreground">
                      {order.type}
                    </span>
                  </div>

                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <div>
                      <span className="text-muted-foreground">Price:</span>{" "}
                      <span className="font-medium">${order.price}</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Amount:</span>{" "}
                      <span className="font-medium">{order.amount} XTZ</span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Filled:</span>{" "}
                      <span className="font-medium">
                        {order.filled} / {order.amount}
                      </span>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Time:</span>{" "}
                      <span className="font-medium">
                        {formatTime(order.timestamp)}
                      </span>
                    </div>
                  </div>
                </div>

                <Button
                  variant="ghost"
                  size="icon"
                  onClick={() => handleCancelOrder(order.id)}
                  className="ml-2 hover:bg-destructive/20 hover:text-destructive"
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>
            </div>
          ))}
        </div>
      )}
    </Card>
  );
};

export default MyOrders;
