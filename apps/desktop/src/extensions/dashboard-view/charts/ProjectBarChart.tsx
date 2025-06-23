import { ResponsiveBar } from "@nivo/bar";
import { useMemo, useRef, useState } from "react";
import { schemeCategory10 } from "d3-scale-chromatic";
import ChartTooltipPortal from "../ChartTooltipPortal";
interface ProjectBarChartProps {
  data: {
    label: string;
    [project: string]: string | number;
  }[];
  keys: string[];
  bucketLabel?: string;
}

const formatTime = (value: number): string => {
  const seconds = Math.round((value as number) * 3600);
  const hrs = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  const padded = (n: number) => String(n).padStart(2, "0");
  const hrsStr = `${hrs}h`;
  const minStr = `${mins}m`;
  const secStr = `${padded(secs)}s`;
  if (hrs > 0) {
    return `${hrsStr} ${minStr} ${secStr}`;
  } else if (mins > 0) {
    return `${minStr} ${secStr}`;
  } else {
    return `${secStr}`;
  }
};

const ProjectBarChart: React.FC<ProjectBarChartProps> = ({
  data,
  keys,
  bucketLabel = "Time",
}) => {
  const [tooltip, setTooltip] = useState<React.ReactNode>(null);
  const [mousePos, setMousePos] = useState<{ x: number; y: number } | null>(
    null,
  );
  const containerRef = useRef<HTMLDivElement>(null);

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

  const keyColorMap = useMemo(() => {
    const colorList = schemeCategory10;
    return Object.fromEntries(
      sortedkeys.map((key, i) => [key, colorList[i % colorList.length]]),
    );
  }, [sortedkeys]);

  return (
    <div
      ref={containerRef}
      className="h-[400px] w-full relative"
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
      onMouseLeave={() => {
        setTooltip(null);
      }}
    >
      <ResponsiveBar
        data={data}
        keys={sortedkeys}
        indexBy="label"
        margin={{ top: 20, right: 30, bottom: 50, left: 50 }}
        padding={0.3}
        groupMode="stacked"
        colors={(bar) => keyColorMap[bar.id as string] || "#888"}
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
        axisLeft={{
          tickSize: 4,
          tickPadding: 6,
          tickRotation: 0,
          legend: "Time (hrs)",
          legendPosition: "middle",
          legendOffset: -40,
        }}
        tooltip={({ data }) => {
          const entries = sortedkeys
            .map((key) => {
              const value = Number(data[key]);
              return value > 0 ? { key, value: value } : null;
            })
            .filter(Boolean) as { key: string; value: number }[];

          const content = (
            <div
              className="rounded-md border border-gray-300 bg-white px-4 py-3 text-sm shadow-lg text-gray-800 min-w-[180px] max-w-[300px] whitespace-nowrap overflow-visible"
              style={{
                transform: "translateX(-10%)",
              }}
            >
              <div className="font-semibold">{data.label}</div>
              {entries.map(({ key, value }) => (
                <div key={key} className="flex items-center gap-2 py-0.5">
                  <span
                    className="w-2.5 h-2.5 rounded-full inline-block"
                    style={{
                      backgroundColor: keyColorMap[key],
                    }}
                  />
                  <span className="flex-1 truncate">{key}</span>
                  <span className="text-gray-600">{formatTime(value)}</span>
                </div>
              ))}
            </div>
          );

          setTooltip(
            <ChartTooltipPortal
              style={{
                top: mousePos?.y ?? 0 + 20,
                left: mousePos?.x ?? 0 + 10,
              }}
            >
              {content}
            </ChartTooltipPortal>,
          );

          return null;
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
      {tooltip}
    </div>
  );
};

export default ProjectBarChart;
