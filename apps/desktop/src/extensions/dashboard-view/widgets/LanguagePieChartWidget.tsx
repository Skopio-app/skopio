import WidgetCard from "../components/WidgetCard";
import CustomPieChart from "@/components/CustomPieChart";
import { useSummaryData } from "../hooks/useSummaryData";
import { Group } from "@/types/tauri.gen";

const LanguagePieChartWidget = () => {
  const options = { groupBy: "language" as Group, mode: "pie" as const };
  const { loading, data } = useSummaryData(options);

  return (
    <WidgetCard title="Languages" loading={loading}>
      <CustomPieChart data={data} />
    </WidgetCard>
  );
};

export default LanguagePieChartWidget;
