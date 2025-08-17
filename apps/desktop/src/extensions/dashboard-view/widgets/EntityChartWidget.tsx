import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "../../../components/StackedBarChart";
import { Group } from "../../../types/tauri.gen";

// const MAX_KEYS = 15;

const EntityChartWidget = () => {
  const options = { groupBy: "entity" as Group, mode: "bar" as const };
  const { loading, data, keys } = useSummaryData(options);

  return (
    <WidgetCard title="Entities" loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default EntityChartWidget;
