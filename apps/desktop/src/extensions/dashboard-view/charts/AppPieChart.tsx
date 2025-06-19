import { ResponsivePie } from "@nivo/pie";

interface AppPieChartProps {
  data: {
    id: string;
    label: string;
    value: number;
    color: string;
  }[];
}

const AppPieChart: React.FC<AppPieChartProps> = ({ data }) => {
  return (
    <div className="h-[300px] w-full">
      <ResponsivePie
        data={data}
        margin={{ top: 10, right: 30, bottom: 60, left: 50 }}
        innerRadius={0.5}
        padAngle={0.6}
        cornerRadius={2}
        activeOuterRadiusOffset={8}
        arcLinkLabelsSkipAngle={10}
        arcLinkLabelsTextColor="#333333"
        arcLinkLabelsThickness={2}
        arcLinkLabelsColor={{ from: "color" }}
        arcLabelsSkipAngle={10}
        arcLabelsTextColor={{ from: "color", modifiers: [["darker", 2]] }}
        legends={[
          {
            anchor: "bottom",
            direction: "row",
            translateY: 56,
            itemWidth: 100,
            itemHeight: 18,
            symbolShape: "circle",
          },
        ]}
      />
    </div>
  );
};

export default AppPieChart;
