import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import CustomPieChart from "../../../components/CustomPieChart";
import { Group } from "../../../types/tauri.gen";

const AppPieChartWidget = () => {
  const options = { groupBy: "app" as Group, mode: "pie" as const };
  const { data, loading } = useSummaryData(options);

  return (
    <WidgetCard title="Apps" loading={loading}>
      <CustomPieChart data={data} />
    </WidgetCard>
  );
};

export default AppPieChartWidget;
