import React from "react";
import { ResponsiveBar } from "@nivo/bar";
import { ResponsiveLine } from "@nivo/line";

interface BasicBarChartProps {
  data: {
    label: string;
    value: number;
  }[];
  goalDuration: number;
}

const BarLineChart: React.FC<BasicBarChartProps> = ({ data, goalDuration }) => {
  if (!data.length) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No data available
      </div>
    );
  }

  const chartMargins = { top: 20, right: 30, bottom: 50, left: 50 };
  const maxBarValue = Math.max(...data.map((d) => d.value));
  const yAxisMaxValue = Math.max(maxBarValue, goalDuration) * 1.1;

  const getBarColor = (bar: any) =>
    bar.data.value >= goalDuration ? "#4ade80" : "#f87171";

  const lineChartData = [
    {
      id: "Goal",
      data: [
        { x: data[0].label, y: goalDuration },
        { x: data[data.length - 1].label, y: goalDuration },
      ],
    },
  ];

  return (
    <div className="h-[220px] w-full relative">
      <div className="absolute inset-0">
        <ResponsiveBar
          data={data}
          indexBy="label"
          keys={["value"]}
          margin={chartMargins}
          padding={0.3}
          valueScale={{ type: "linear", min: 0, max: yAxisMaxValue }}
          indexScale={{ type: "band", round: true }}
          axisBottom={{
            tickSize: 4,
            tickPadding: 6,
            tickRotation: 0,
            // legend: "Day",
            legendPosition: "middle",
            legendOffset: 36,
          }}
          axisLeft={{
            tickSize: 4,
            tickPadding: 6,
            // legend: "Time (hrs)",
            legendPosition: "middle",
            legendOffset: -40,
          }}
          enableLabel={false}
          colors={getBarColor}
          role="application"
          layers={["grid", "axes", "bars"]}
        />
      </div>

      <div className="absolute inset-0 z-10 pointer-events-none">
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
