import { ResponsiveBar } from "@nivo/bar";
import { useEffect, useMemo, useRef, useState } from "react";
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
  const mousePosRef = useRef<{ x: number; y: number } | null>(null);
  const rafId = useRef<number | null>(null);
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

  const tooltipEntriesMap = useMemo(() => {
    const map: Record<string, { key: string; value: number }[]> = {};

    for (const bar of data) {
      const entries = sortedkeys
        .map((key) => {
          const value = Number(bar[key] ?? 0);
          return value > 0 ? { key, value } : null;
        })
        .filter(Boolean) as { key: string; value: number }[];
      map[bar.label] = entries;
    }

    return map;
  }, [data, sortedkeys]);

  const colorCache = useMemo(() => {
    const cache: Record<string, string> = {};
    for (const key of keys) {
      cache[key] = getColor({ id: key });
    }
    return cache;
  }, [keys, getColor]);

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
    const x = e.clientX + 5;
    const y = e.clientY + 5;

    mousePosRef.current = { x, y };

    if (rafId.current === null) {
      rafId.current = requestAnimationFrame(() => {
        if (mousePosRef.current) {
          setMousePos(mousePosRef.current);
        }
        rafId.current = null;
      });
    }
  };

  useEffect(() => {
    return () => {
      if (rafId.current !== null) {
        cancelAnimationFrame(rafId.current);
      }
    };
  });

  const MAX_TOOLTIP_ENTRIES = 10;

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
      onMouseMove={handleMouseMove}
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
          const entries = [...(tooltipEntriesMap[data.label] ?? [])]
            .sort((a, b) => b.value - a.value)
            .slice(0, MAX_TOOLTIP_ENTRIES);

          if (!mousePos) return null;

          return (
            <ChartTooltipPortal
              style={{
                top: mousePos.y,
                left: mousePos.x,
                zIndex: 50,
              }}
            >
              <div className="max-h-96 scroll-hidden overflow-y-auto rounded-md border border-gray-300 bg-white px-4 py-3 text-sm shadow-lg text-gray-800 min-w-[200px] max-w-[320px]">
                <div className="font-semibold mb-1">{data.label}</div>
                {entries.map(({ key, value }) => (
                  <div key={key} className="flex items-center gap-2 py-0.5">
                    <span
                      className="w-2.5 h-2.5 rounded-full inline-block shrink-0"
                      style={{ backgroundColor: colorCache[key] }}
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
        enableLabel={false}
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
