import { ResponsiveBarCanvas } from "@nivo/bar";
import { useRef, useState, useEffect, useMemo, useCallback } from "react";
import ChartTooltipPortal from "@/components/ChartTooltipPortal";
import { formatDuration } from "@/utils/time";
import { useChartColor, useCssVarColor } from "@/hooks/useChartColor";
import { truncateValue } from "@/utils/data";
import { BarChartData } from "@/types/chart";

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

const MAX_TOOLTIP_ENTRIES = 10;

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
  const { getColorForKey } = useChartColor();
  const axisTextColor = useCssVarColor("--muted-foreground");
  const gridColor = useCssVarColor("--input");

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

  const entriesByLabel = useMemo(() => {
    const map = new Map<
      string,
      Array<{ key: string; value: number; displayKey: string }>
    >();

    for (const row of data) {
      const label = row.label;
      const entries = keys
        .map((k) => ({ key: k, value: Number(row[k] ?? 0) }))
        .filter(({ value }) => value > 0)
        .sort((a, b) => b.value - a.value)
        .slice(0, MAX_TOOLTIP_ENTRIES)
        .map(({ key, value }) => ({
          key,
          value,
          displayKey: truncateValue(key, 25),
        }));

      map.set(label, entries);
    }
    return map;
  }, [data, keys]);

  const renderTooltip = useCallback(
    ({ data: pointData }: { data: BarChartData }) => {
      if (!mousePos) return null;

      const label = pointData.label;
      const entries = entriesByLabel.get(label) ?? [];

      return (
        <ChartTooltipPortal
          style={{
            top: mousePos.y,
            left: mousePos.x,
            zIndex: 50,
          }}
        >
          <div className="max-h-96 overflow-y-auto rounded-md border border-border bg-background px-4 py-3 text-sm shadow-lg text-foreground min-w-[200px] max-w-[320px]">
            <div className="font-semibold mb-1">{label}</div>
            {entries.map(({ value, displayKey }) => {
              return (
                <div
                  key={displayKey}
                  className="flex items-center gap-2 py-0.5"
                >
                  <span
                    className="w-2.5 h-2.5 rounded-full inline-block shrink-0"
                    style={{
                      backgroundColor: getColorForKey(displayKey),
                    }}
                  />
                  <span className="flex-1 text-xs">{displayKey}</span>
                  <span className="text-muted-foreground text-xs">
                    {formatDuration(value)}
                  </span>
                </div>
              );
            })}
          </div>
        </ChartTooltipPortal>
      );
    },
    [entriesByLabel, getColorForKey, mousePos],
  );

  // TODO: Reuse not available text
  if (!data.length) {
    return (
      <p className="h-[220px] w-full flex items-center justify-center text-sm text-muted-foreground">
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
        colors={(bar) => getColorForKey(String(bar.id))}
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
        theme={{
          axis: {
            domain: {
              line: {
                stroke: axisTextColor,
                strokeWidth: 1,
              },
            },
            ticks: {
              line: {
                stroke: axisTextColor,
                strokeWidth: 1,
              },
              text: {
                fill: axisTextColor,
                fontSize: 11,
              },
            },
            legend: {
              text: {
                fill: axisTextColor,
                fontSize: 12,
                fontWeight: 500,
              },
            },
          },
          grid: {
            line: {
              stroke: gridColor,
              strokeWidth: 1,
              strokeDasharray: "2,2",
            },
          },
        }}
        tooltip={renderTooltip}
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
