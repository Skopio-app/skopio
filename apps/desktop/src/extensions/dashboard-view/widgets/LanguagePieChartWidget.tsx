import WidgetCard from "../components/WidgetCard";
import CustomPieChart from "@/components/CustomPieChart";
import { useSummaryData } from "../hooks/useSummaryData";
import { Group } from "@/types/tauri.gen";

const LanguagePieChartWidget = () => {
  const options = { groupBy: "language" as Group, mode: "pie" as const };
  const { loading, data } = useSummaryData(options);

  return (
    <WidgetCard
      tooltip="The duration of use of the various detected programming languages."
      title="Languages"
      loading={loading}
    >
      <CustomPieChart data={data} />
    </WidgetCard>
  );
};

export default LanguagePieChartWidget;
