import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import CustomPieChart from "@/components/CustomPieChart";
import { Group } from "@/types/tauri.gen";

const AppPieChartWidget = () => {
  const options = { groupBy: "app" as Group, mode: "pie" as const };
  const { data, loading } = useSummaryData(options);

  return (
    <WidgetCard
      tooltip="The duration of use of the various tracked apps."
      title="Apps"
      loading={loading}
      skeletonVariant="pie"
    >
      <CustomPieChart data={data} />
    </WidgetCard>
  );
};

export default AppPieChartWidget;
