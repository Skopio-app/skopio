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

interface SkeletonCalendarChartProps {
  weeks?: number;
}

const SkeletonCalendarChart: React.FC<SkeletonCalendarChartProps> = ({
  weeks = 14,
}) => {
  // 7 days per week
  const totalCells = weeks * 7;

  const shade = (i: number) => {
    const x = Math.sin(i * 9377) * 10000;
    const frac = x - Math.floor(x);
    // opacity range: 0.25..0.85
    return 0.25 + frac * 0.6;
  };

  return (
    <div className="h-[200px] w-full px-4 py-3 animate-pulse">
      {/* calendar body */}
      <div className="flex w-full">
        {/* grid */}
        <div className="flex-1 overflow-hidden">
          <div
            className="grid gap-[2px]"
            style={{
              gridTemplateColumns: `repeat(${weeks}, minmax(0, 1fr))`,
              gridTemplateRows: "repeat(7, minmax(0, 1fr))",
              height: 120,
            }}
          >
            {Array.from({ length: totalCells }).map((_, i) => (
              <div
                key={i}
                className="rounded-[5px] bg-gray-300/60"
                style={{
                  opacity: shade(i),
                }}
              />
            ))}
          </div>
        </div>

        <div className="w-8 shrink-0" />
      </div>
    </div>
  );
};

const SkeletonChart: React.FC<SkeletonChartProps> = ({ variant = "bar" }) => {
  if (variant === "calendar") return <SkeletonCalendarChart />;
  return <SkeletonBarChart />;
};

export default SkeletonChart;
