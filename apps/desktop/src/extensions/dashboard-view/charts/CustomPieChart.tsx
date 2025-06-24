import { ResponsivePie } from "@nivo/pie";
import { useMemo } from "react";
import { formatDuration } from "../dateRanges";
import { useOrdinalColorScale } from "@nivo/colors";

interface CustomPieChartProps {
  data: {
    id: string;
    label: string;
    value: number;
  }[];
}

const CustomPieChart: React.FC<CustomPieChartProps> = ({ data }) => {
  const chartData = useMemo(() => data, [data]);
  const getColor = useOrdinalColorScale({ scheme: "nivo" }, "id");

  // TODO: Reuse the following placeholder text
  if (!chartData.length) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No data available
      </div>
    );
  }

  return (
    <div className="h-[200px] w-full flex">
      <div className="flex-1">
        <ResponsivePie
          data={chartData}
          margin={{ top: 30, right: 10, left: 20, bottom: 30 }}
          innerRadius={0.5}
          padAngle={0.6}
          cornerRadius={2}
          activeOuterRadiusOffset={8}
          arcLinkLabelsSkipAngle={5}
          arcLinkLabelsTextColor="#333333"
          arcLinkLabelsThickness={2}
          colors={{ scheme: "nivo" }}
          arcLinkLabelsDiagonalLength={12}
          arcLinkLabelsColor={{ from: "color" }}
          arcLabelsSkipAngle={10}
          arcLabelsTextColor={{ from: "color", modifiers: [["darker", 2]] }}
          legends={[]}
          enableArcLabels={false}
          tooltip={({ datum }) => {
            const time = formatDuration(datum.value);

            return (
              <div className="rounded-md border border-gray-300 bg-white px-3 py-2 text-sm shadow-md text-gray-600 flex items-center gap-2">
                <span
                  className="w-2.5 h-2.5 rounded-full inline-block"
                  style={{ backgroundColor: datum.color }}
                />
                <div className="flex flex-col">
                  <span className="font-semibold">{datum.id}</span>
                  <span className="text-xs text-gray-600">{time}</span>
                </div>
              </div>
            );
          }}
        />
      </div>

      <div className="w-52 pr-2 pl-3 overflow-y-auto max-h-[300px] space-y-2 text-sm scroll-hidden">
        {data.map((d) => (
          <div key={d.id} className="flex items-center justify-between">
            <div className="flex items-center gap-2 min-w-0">
              <span
                className="w-3 h-3 rounded-full inline-block shrink-0"
                style={{ backgroundColor: getColor(d) }}
              />
              <span className="truncate text-gray-700 text-xs max-w-[7rem]">
                {d.label}
              </span>
            </div>
            <span className="truncate text-xs text-gray-500 max-w[5rem] text-right">
              {formatDuration(d.value)}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default CustomPieChart;
