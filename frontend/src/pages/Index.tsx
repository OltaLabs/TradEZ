import AnimatedBackground from "@/components/AnimatedBackground";
import Header from "@/components/Header";
import TradingChart from "@/components/TradingChart";
import OrderBook from "@/components/OrderBook";
import OrderForm from "@/components/OrderForm";
import MyOrders from "@/components/MyOrders";

const Index = () => {
  return (
    <div className="min-h-screen bg-background relative">
      <AnimatedBackground />
      
      <div className="relative z-10">
        <Header />
        
        <main className="container mx-auto px-4 py-6">
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-4">
            {/* Left side - Chart */}
            <div className="lg:col-span-7">
              <TradingChart />
            </div>

            {/* Middle - Order Book */}
            <div className="lg:col-span-3">
              <OrderBook />
            </div>

            {/* Right side - Order Form */}
            <div className="lg:col-span-2">
              <OrderForm />
            </div>
          </div>

          {/* Bottom - My Orders */}
          <div className="mt-4">
            <MyOrders />
          </div>
        </main>
      </div>
    </div>
  );
};

export default Index;
