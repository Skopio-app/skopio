import ChartContainer from "../ChartContainer";
import StackedBarChart from "@/components/StackedBarChart";
import { useProjectSummaryData } from "../../hooks/useProjectSummaryData";
import { Group } from "@/types/tauri.gen";

const CategoryChartSection = () => {
  const options = { groupBy: "category" as Group, mode: "bar" as const };
  const { loading, data, keys } = useProjectSummaryData(options);

  return (
    <ChartContainer loading={loading}>
      <StackedBarChart
        data={data}
        keys={keys}
        axisBottom={false}
        axisLeft={true}
        bucketLabel="Time"
      />
    </ChartContainer>
  );
};

export default CategoryChartSection;
