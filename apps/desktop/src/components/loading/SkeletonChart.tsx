export type SkeletonChartVariant = "bar" | "calendar" | "pie";

interface SkeletonChartProps {
  variant?: SkeletonChartVariant;
}

interface SkeletonCalendarChartProps {
  weeks?: number;
}

interface SkeletonPieChartProps {
  legendRows?: number;
  showLegend?: boolean;
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

const SkeletonPieChart: React.FC<SkeletonPieChartProps> = ({
  legendRows = 6,
  showLegend = true,
}) => {
  const widthFrac = (i: number) => {
    const x = Math.sin(i * 9377) * 10000;
    const frac = x - Math.floor(x);
    // 40%..95%
    return 0.4 + frac * 0.55;
  };

  return (
    <div className="h-[200px] w-full flex items-center gap-4 px-4 animate-pulse">
      {/* Donut placeholder */}
      <div className="flex-1 flex items-center justify-center min-w-0">
        <div className="relative w-[140px] h-[140px] rounded-full bg-gray-300/50">
          {/* donut hole */}
          <div className="absolute inset-0 m-auto w-[70px] h-[70px] rounded-full bg-background" />
          {/* subtle “slice” hints */}
          <div className="absolute inset-0 rounded-full border border-gray-300/40" />
        </div>
      </div>

      {showLegend && (
        <div className="w-52 pr-2 pl-1 space-y-2">
          {Array.from({ length: legendRows }).map((_, i) => (
            <div key={i} className="flex items-center justify-between gap-2">
              <div className="flex items-center gap-2 min-w-0 flex-1">
                <div className="w-3 h-3 rounded-xl bg-gray-300/60 shrink-0" />
                <div
                  className="h-3 rounded bg-gray-300/45"
                  style={{ width: `${Math.round(widthFrac(i) * 100)}%` }}
                />
              </div>
              <div className="h-3 w-12 rounded bg-gray-300/30 shrink-0" />
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

const SkeletonChart: React.FC<SkeletonChartProps> = ({ variant = "bar" }) => {
  if (variant === "calendar") return <SkeletonCalendarChart />;
  if (variant === "pie") return <SkeletonPieChart />;
  return <SkeletonBarChart />;
};

export default SkeletonChart;
