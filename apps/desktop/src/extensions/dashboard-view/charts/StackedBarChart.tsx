import { ResponsiveBar } from "@nivo/bar";
import { useMemo, useRef, useState } from "react";
import ChartTooltipPortal from "../ChartTooltipPortal";
import { formatDuration } from "../dateRanges";
import { useOrdinalColorScale } from "@nivo/colors";
interface StackedBarChartProps {
  data: {
    label: string;
    [project: string]: string | number;
  }[];
  keys: string[];
  bucketLabel?: string;
}

const StackedBarChart: React.FC<StackedBarChartProps> = ({
  data,
  keys,
  bucketLabel = "Time",
}) => {
  const [mousePos, setMousePos] = useState<{ x: number; y: number } | null>(
    null,
  );
  const containerRef = useRef<HTMLDivElement>(null);
  const getColor = useOrdinalColorScale({ scheme: "nivo" }, "id");

  const sortedkeys = useMemo(() => {
    const totals: Record<string, number> = {};

    for (const entry of data) {
      for (const key of keys) {
        const val = Number(entry[key] ?? 0);
        totals[key] = (totals[key] ?? 0) + val;
      }
    }

    return [...keys].sort((a, b) => totals[a] - totals[b]);
  }, [data, keys]);

  if (!data.length) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No data available
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="h-[200px] w-full relative"
      onMouseMove={(e) => {
        if (!containerRef.current) return;
        const rect = containerRef.current?.getBoundingClientRect();

        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;

        setMousePos({
          x,
          y,
        });
      }}
    >
      <ResponsiveBar
        data={data}
        keys={sortedkeys}
        indexBy="label"
        margin={{ top: 20, right: 30, bottom: 50, left: 50 }}
        padding={0.3}
        groupMode="stacked"
        colors={(bar) => getColor({ id: bar.id })}
        borderRadius={2}
        borderWidth={1}
        borderColor={{ from: "color", modifiers: [["darker", 1.6]] }}
        axisBottom={{
          tickSize: 4,
          tickPadding: 6,
          tickRotation: 0,
          legend: bucketLabel,
          legendPosition: "middle",
          legendOffset: 36,
        }}
        axisLeft={null}
        tooltip={({ data }) => {
          const entries = sortedkeys
            .map((key) => {
              const value = Number(data[key]);
              return value > 0 ? { key, value } : null;
            })
            .filter(Boolean) as { key: string; value: number }[];

          if (!mousePos) return null;

          return (
            <ChartTooltipPortal
              style={{
                top: mousePos.y + 20,
                left: mousePos.x + 20,
                zIndex: 50,
              }}
            >
              <div className="max-h-96 scroll-hidden overflow-y-auto rounded-md border border-gray-300 bg-white px-4 py-3 text-sm shadow-lg text-gray-800 min-w-[200px] max-w-[320px]">
                <div className="font-semibold mb-1">{data.label}</div>
                {entries.map(({ key, value }) => (
                  <div key={key} className="flex items-center gap-2 py-0.5">
                    <span
                      className="w-2.5 h-2.5 rounded-full inline-block shrink-0"
                      style={{ backgroundColor: getColor({ id: key }) }}
                    />
                    <span className="truncate flex-1 text-xs">{key}</span>
                    <span className="text-gray-500 text-xs">
                      {formatDuration(value)}
                    </span>
                  </div>
                ))}
              </div>
            </ChartTooltipPortal>
          );
        }}
        label={""}
        labelSkipWidth={12}
        labelSkipHeight={12}
        labelTextColor={{
          from: "color",
          modifiers: [["darker", 1.6]],
        }}
        animate
        motionConfig="gentle"
        role="application"
        ariaLabel="Time bucket project summary chart"
      />
    </div>
  );
};

export default StackedBarChart;
