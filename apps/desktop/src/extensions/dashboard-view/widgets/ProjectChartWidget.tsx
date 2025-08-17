import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "../../../components/StackedBarChart";
import { Group } from "../../../types/tauri.gen";

const ProjectChartWidget = () => {
  const options = { groupBy: "project" as Group, mode: "bar" as const };
  const { data, loading, keys } = useSummaryData(options);

  return (
    <WidgetCard title="Projects" loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default ProjectChartWidget;
