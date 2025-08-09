import React from "react";
import { ResponsiveBar } from "@nivo/bar";
import { ResponsiveLine } from "@nivo/line";
import { formatDuration } from "../../../utils/time";
import { TimeSpan } from "../../../types/tauri.gen";

interface BasicBarChartProps {
  data: {
    label: string;
    value: number;
  }[];
  goalDuration: number;
  timeSpan: TimeSpan;
}

const BarLineChart: React.FC<BasicBarChartProps> = ({
  data,
  goalDuration,
  timeSpan,
}) => {
  if (!data.length) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No data available
      </div>
    );
  }

  const sortedData = [...data].sort((a, b) => a.label.localeCompare(b.label));

  const chartMargins = { top: 20, right: 30, bottom: 50, left: 100 };
  const maxBarValue = Math.max(...sortedData.map((d) => d.value));
  const yAxisMaxValue = Math.max(maxBarValue, goalDuration) * 1.1;

  const getBarColor = (bar: any) =>
    bar.data.value >= goalDuration ? "#4ade80" : "#f87171";

  const lineChartData = [
    {
      id: "Goal",
      data: [
        { x: sortedData[0].label, y: goalDuration },
        { x: sortedData[sortedData.length - 1].label, y: goalDuration },
      ],
    },
  ];

  return (
    <div className="h-[220px] w-full relative">
      <div className="absolute inset-0">
        <ResponsiveBar
          data={sortedData}
          indexBy="label"
          keys={["value"]}
          margin={chartMargins}
          padding={0.3}
          valueScale={{ type: "linear", min: 0, max: yAxisMaxValue }}
          indexScale={{ type: "band", round: true }}
          tooltip={({ value, indexValue, color }) => {
            const actualTime = formatDuration(value);
            const goalTime = formatDuration(goalDuration);

            let formattedLabel = indexValue;
            const date = new Date(indexValue);

            if (!Number.isNaN(date.getTime()) && timeSpan === "day") {
              const dayName = date.toLocaleDateString(undefined, {
                weekday: "short",
              });
              formattedLabel = `${dayName}, ${indexValue}`;
            }

            return (
              <div className="min-w-48 rounded-md border border-gray-200 bg-white px-3 py-2 text-sm shadow-md text-gray-700">
                <div className="flex items-center gap-2 mb-1">
                  <span
                    className="w-2.5 h-2.5 rounded-full inline-block"
                    style={{ backgroundColor: color }}
                  />
                  <span className="font-semibold">{formattedLabel}</span>
                </div>
                <div className="text-xs space-y-1">
                  <p>Actual: {actualTime}</p>
                  <p>Goal: {goalTime}</p>
                </div>
              </div>
            );
          }}
          axisBottom={{
            tickSize: 4,
            tickPadding: 6,
            tickValues: 5,
            tickRotation: 0,
            legendPosition: "middle",
            legendOffset: 36,
          }}
          axisLeft={{
            tickSize: 4,
            tickPadding: 6,
            tickRotation: 0,
            tickValues: 5,
            legend: "Time",
            legendPosition: "middle",
            legendOffset: -80,
            format: (value) => formatDuration(value),
          }}
          enableLabel={false}
          colors={getBarColor}
          role="application"
          layers={["grid", "axes", "bars"]}
        />
      </div>

      <div className="absolute inset-0 pointer-events-none">
        <ResponsiveLine
          data={lineChartData}
          margin={chartMargins}
          xScale={{ type: "point" }}
          yScale={{ type: "linear", min: 0, max: yAxisMaxValue }}
          curve="linear"
          enablePoints={true}
          pointSize={6}
          pointColor="#3b82f6"
          pointBorderWidth={2}
          pointBorderColor="#3b82f6"
          lineWidth={2}
          colors={["#3b82f6"]}
          useMesh={false}
          enableGridX={false}
          enableGridY={false}
          axisTop={null}
          axisRight={null}
          axisBottom={null}
          axisLeft={null}
        />
      </div>
    </div>
  );
};

export default BarLineChart;
