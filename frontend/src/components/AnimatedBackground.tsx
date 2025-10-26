import { Settings } from "lucide-react";

const AnimatedBackground = () => {
  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden">
      <Settings className="gear gear-1 w-64 h-64 text-primary/10" />
      <Settings className="gear gear-2 w-48 h-48 text-primary/10" />
      <Settings className="gear gear-3 w-56 h-56 text-primary/10" />
      <Settings className="gear gear-4 w-40 h-40 text-primary/10" />
      
      {/* Gradient overlays */}
      <div className="absolute top-0 left-0 w-full h-full bg-[radial-gradient(circle_at_20%_50%,hsl(var(--primary)/0.1),transparent_50%)]" />
      <div className="absolute top-0 right-0 w-full h-full bg-[radial-gradient(circle_at_80%_30%,hsl(var(--buy)/0.08),transparent_50%)]" />
    </div>
  );
};

export default AnimatedBackground;
