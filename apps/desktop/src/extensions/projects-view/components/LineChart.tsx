import { PointTooltipProps, ResponsiveLine } from "@nivo/line";
import { formatDuration } from "@/utils/time";
import ChartTooltipPortal from "@/components/ChartTooltipPortal";
import { useEffect, useRef, useState } from "react";

interface LineChartProps {
  id: string;
  data: {
    x: string;
    y: number;
  }[];
}

const LineChart: React.FC<LineChartProps> = ({ id, data }) => {
  const [mousePos, setMousePos] = useState<{ x: number; y: number } | null>(
    null,
  );
  const mousePosRef = useRef<{ x: number; y: number } | null>(null);
  const rafId = useRef<number | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);

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

  if (!data.length) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No data available
      </div>
    );
  }
  const sortedData = [...data].sort((a, b) => a.x.localeCompare(b.x));
  const lineChartData = [
    {
      id: id,
      data: sortedData.map((point) => ({ x: point.x, y: point.y })),
    },
  ];
  const maxChartValue = Math.max(...sortedData.map((d) => d.y));
  const yAxisMaxValue = maxChartValue * 1.1;

  return (
    <div
      ref={containerRef}
      className="h-[200px] w-full flex"
      onMouseMove={handleMouseMove}
    >
      <div className="flex-1">
        <ResponsiveLine
          data={lineChartData}
          margin={{
            top: 20,
            right: 30,
            bottom: 50,
            left: 100,
          }}
          xScale={{ type: "point" }}
          yScale={{
            type: "linear",
            min: 0,
            max: yAxisMaxValue,
            stacked: false,
          }}
          tooltip={({ point }: PointTooltipProps<LineChartProps>) => {
            const formattedTime = formatDuration(point.data.y);
            let formattedLabel = point.data.x;
            const date = new Date(point.data.x);

            if (!Number.isNaN(date.getTime())) {
              const dayName = date.toLocaleDateString(undefined, {
                weekday: "short",
              });
              formattedLabel = `${dayName}, ${point.data.x}`;
            }

            if (!mousePos) return null;

            return (
              <ChartTooltipPortal
                style={{
                  top: mousePos.y,
                  left: mousePos.x,
                  zIndex: 50,
                }}
              >
                <div className="min-w-32 rounded-md border border-gray-200 bg-white px-3 py-2 text-sm shadow-md text-neutral-700">
                  <h3 className="font-medium text-xs">{formattedLabel}</h3>
                  <p className="text-xs">{formattedTime}</p>
                </div>
              </ChartTooltipPortal>
            );
          }}
          curve="cardinal"
          enablePoints={true}
          pointSize={6}
          pointBorderWidth={2}
          lineWidth={2}
          useMesh={true}
          enableArea={true}
          areaOpacity={0.2}
          enableGridX={false}
          enableGridY={true}
          axisTop={null}
          axisRight={null}
          axisBottom={null}
          axisLeft={{
            tickSize: 5,
            tickPadding: 5,
            tickRotation: 0,
            tickValues: 5,
            legend: "Time",
            legendOffset: -80,
            legendPosition: "middle",
            format: (value) => formatDuration(value),
          }}
        />
      </div>
    </div>
  );
};

export default LineChart;
