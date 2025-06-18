import { ResponsiveBar } from "@nivo/bar";

export interface ProjectBarChartProps {
  data: {
    week: string;
    [project: string]: string | number;
  }[];
  keys: string[];
}

const ProjectBarChart: React.FC<ProjectBarChartProps> = ({ data, keys }) => {
  return (
    <div className="h-[300px] w-full">
      <ResponsiveBar
        data={data}
        keys={keys}
        indexBy="week"
        margin={{ top: 20, right: 30, bottom: 50, left: 50 }}
        padding={0.3}
        groupMode="stacked"
        colors={{ scheme: "nivo" }}
        borderRadius={2}
        borderWidth={1}
        borderColor={{ from: "color", modifiers: [["darker", 1.6]] }}
        axisBottom={{
          tickSize: 4,
          tickPadding: 6,
          tickRotation: 0,
          legend: "Week",
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
        labelSkipWidth={12}
        labelSkipHeight={12}
        labelTextColor={{
          from: "color",
          modifiers: [["darker", 1.6]],
        }}
        animate
        motionConfig="gentle"
        role="application"
        ariaLabel="Weekly project time chart"
      />
    </div>
  );
};

export default ProjectBarChart;
