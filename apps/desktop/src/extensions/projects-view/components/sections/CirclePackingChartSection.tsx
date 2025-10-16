import { Group } from "@/types/tauri.gen";
import ChartContainer from "../ChartContainer";
import CirclePackingChart from "../CirclePackingChart";
import { useProjectSummaryData } from "../../hooks/useProjectSummaryData";

const CirclePackingChartSection = () => {
  const options = { groupBy: "entity" as Group, mode: "list" as const };
  const { data, loading } = useProjectSummaryData(options);

  return (
    <ChartContainer loading={loading}>
      <CirclePackingChart data={data} />
    </ChartContainer>
  );
};

export default CirclePackingChartSection;
