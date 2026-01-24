export type SkeletonChartVariant = "bar" | "calendar";

interface SkeletonChartProps {
  variant?: SkeletonChartVariant;
}

const SkeletonBarChart = () => {
  const barHeight = (i: number) => {
    const x = Math.sin(i * 9377) * 10000;
    const frac = x - Math.floor(x);
    return 40 + Math.round(frac * 100);
  };

  return (
    <div className="h-[200px] w-full relative flex gap-1 items-end px-4 animate-pulse">
      {Array.from({ length: 12 }).map((_, idx) => (
        <div
          key={idx}
          className="flex-1 bg-gray-300/60 rounded-md"
          style={{
            height: `${barHeight(idx)}px`,
            minWidth: "6px",
          }}
        />
      ))}
    </div>
  );
};

const SkeletonCalendarChart = () => {
  const cols = 7;
  const rows = 6;

  const intensity = (i: number) => {
    const x = Math.sin(i * 9377) * 10000;
    const frac = x - Math.floor(x);
    return 0.25 + frac * 0.55;
  };

  return (
    <div className="h-[200px] w-full p-4 animate-pulse">
      <div className="flex items-center justify-between mb-3">
        <div className="h-4 w-28 bg-gray-300/60 rounded-md" />
        <div className="h-4 w-16 bg-gray-300/40 rounded-md" />
      </div>

      {/* weekday row */}
      <div className="grid grid-cols-7 gap-2 mb-2">
        {Array.from({ length: 7 }).map((_, i) => (
          <div key={i} className="h-3 bg-gray-300/30 rounded-md" />
        ))}
      </div>

      {/* calendar grid */}
      <div className="grid grid-cols-7 gap-2">
        {Array.from({ length: cols * rows }).map((_, i) => (
          <div
            key={i}
            className="aspect-square rounded-md bg-gray-300/60"
            style={{
              opacity: intensity(i),
              minWidth: "10px",
            }}
          />
        ))}
      </div>
    </div>
  );
};

const SkeletonChart: React.FC<SkeletonChartProps> = ({ variant = "bar" }) => {
  if (variant === "calendar") return <SkeletonCalendarChart />;
  return <SkeletonBarChart />;
};

export default SkeletonChart;
