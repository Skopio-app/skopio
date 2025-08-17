import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "../../../components/StackedBarChart";
import { Group } from "../../../types/tauri.gen";

const CategoryChartWidget = () => {
  const options = { groupBy: "category" as Group, mode: "bar" as const };
  const { loading, data, keys } = useSummaryData(options);

  return (
    <WidgetCard title="Categories" loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default CategoryChartWidget;
