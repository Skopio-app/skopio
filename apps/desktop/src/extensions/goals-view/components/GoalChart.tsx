import { ResponsiveBar } from "@nivo/bar";

interface BasicBarChartProps {
  data: {
    label: string;
    value: number;
  }[];
  goalDuration: number;
}

const BasicBarChart: React.FC<BasicBarChartProps> = ({
  data,
  goalDuration,
}) => {
  if (!data.length) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No data available
      </div>
    );
  }

  const getColor = (bar: any) =>
    bar.data.value >= goalDuration ? "#4ade80" : "#f87171";

  return (
    <div className="h-[200px] w-full relative">
      <ResponsiveBar
        data={data}
        indexBy="label"
        margin={{ top: 20, right: 30, bottom: 50, left: 50 }}
        padding={0.3}
        borderRadius={2}
        borderWidth={1}
        borderColor={{ from: "color", modifiers: [["darker", 1.6]] }}
        axisBottom={{
          tickSize: 4,
          tickPadding: 6,
          tickRotation: 0,
          legend: "",
          legendPosition: "middle",
          legendOffset: 36,
        }}
        enableLabel={false}
        labelSkipWidth={12}
        labelSkipHeight={12}
        colors={(bar) => getColor(bar)}
        labelTextColor={{
          from: "color",
          modifiers: [["darker", 1.6]],
        }}
        role="application"
      />
    </div>
  );
};

export default BasicBarChart;
