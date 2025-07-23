import { ResponsiveBarCanvas } from "@nivo/bar";
import { useRef, useState, useEffect } from "react";
import { useOrdinalColorScale } from "@nivo/colors";
import { useColorCache } from "../stores/useColorCache";
import ChartTooltipPortal from "./ChartTooltipPortal";
import { formatDuration } from "../utils/time";

interface StackedBarChartProps {
  data: {
    label: string;
    [key: string]: string | number;
  }[];
  keys: string[];
  bucketLabel?: string;
  axisBottom?: boolean;
  axisLeft?: boolean;
}

const StackedBarChart: React.FC<StackedBarChartProps> = ({
  data,
  keys,
  bucketLabel = "Time",
  axisBottom = true,
  axisLeft = false,
}) => {
  const [mousePos, setMousePos] = useState<{ x: number; y: number } | null>(
    null,
  );
  const mousePosRef = useRef<{ x: number; y: number } | null>(null);
  const rafId = useRef<number | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  const getColor = useOrdinalColorScale({ scheme: "nivo" }, "id");

  const getColorForKey = (
    key: string,
    getColorScale: (input: { id: string }) => string,
  ): string => {
    const cachedColor = useColorCache.getState().getColor(key);
    if (cachedColor) return cachedColor;
    const newColor = getColorScale({ id: key });
    useColorCache.getState().setColor(key, newColor);
    return newColor;
  };

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
  }, []);

  const MAX_TOOLTIP_ENTRIES = 10;

  // TODO: Reuse not available text
  if (!data.length) {
    return (
      <p className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No data available
      </p>
    );
  }

  return (
    <div
      ref={containerRef}
      className="h-[200px] w-full relative"
      onMouseMove={handleMouseMove}
    >
      <ResponsiveBarCanvas
        data={data}
        keys={keys}
        indexBy="label"
        margin={{ top: 20, right: 30, bottom: 50, left: axisLeft ? 90 : 50 }}
        padding={0.3}
        groupMode="stacked"
        colors={(bar) => getColorForKey(String(bar.id), getColor)}
        borderRadius={2}
        borderWidth={1}
        borderColor={{ from: "color", modifiers: [["darker", 1.6]] }}
        axisBottom={
          axisBottom
            ? {
                tickSize: 4,
                tickPadding: 6,
                tickRotation: 0,
                legend: bucketLabel,
                legendPosition: "middle",
                legendOffset: 36,
              }
            : null
        }
        axisLeft={
          axisLeft
            ? {
                tickSize: 10,
                tickPadding: 5,
                tickRotation: 0,
                tickValues: 5,
                legend: bucketLabel,
                legendOffset: -80,
                legendPosition: "middle",
                format: (value) => formatDuration(value),
              }
            : null
        }
        tooltip={({ data }) => {
          const entries = Object.entries(data)
            .filter(([key]) => key !== "label" && typeof data[key] === "number")
            .map(([key, value]) => ({
              key,
              value: Number(value),
            }))
            .filter(({ value }) => value > 0)
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
              <div className="max-h-96 overflow-y-auto rounded-md border border-gray-300 bg-white px-4 py-3 text-sm shadow-lg text-gray-800 min-w-[200px] max-w-[320px]">
                <div className="font-semibold mb-1">{data.label}</div>
                {entries.map(({ key, value }) => (
                  <div key={key} className="flex items-center gap-2 py-0.5">
                    <span
                      className="w-2.5 h-2.5 rounded-full inline-block shrink-0"
                      style={{ backgroundColor: getColorForKey(key, getColor) }}
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
        role="application"
      />
    </div>
  );
};

export default StackedBarChart;
