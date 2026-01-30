import { Group } from "@/types/tauri.gen";
import ChartContainer from "../ChartContainer";
import CustomPieChart from "@/components/CustomPieChart";
import { useProjectSummaryData } from "../../hooks/useProjectSummaryData";

const LanguagePieChartSection = () => {
  const options = { groupBy: "language" as Group, mode: "pie" as const };

  const { data, loading } = useProjectSummaryData(options);

  return (
    <ChartContainer title="Languages" loading={loading} skeletonVariant="pie">
      <CustomPieChart data={data} />
    </ChartContainer>
  );
};

export default LanguagePieChartSection;
