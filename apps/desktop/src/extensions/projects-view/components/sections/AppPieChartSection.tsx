import { Group } from "@/types/tauri.gen";
import ChartContainer from "../ChartContainer";
import CustomPieChart from "@/components/CustomPieChart";
import { useProjectSummaryData } from "../../hooks/useProjectSummaryData";

const AppPieChartSection = () => {
  const options = { groupBy: "app" as Group, mode: "pie" as const };
  const { data, loading } = useProjectSummaryData(options);

  return (
    <ChartContainer title="Apps" loading={loading} skeletonVariant="pie">
      <CustomPieChart data={data} />
    </ChartContainer>
  );
};

export default AppPieChartSection;
