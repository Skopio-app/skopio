import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "../../../components/StackedBarChart";
import { Group } from "../../../types/tauri.gen";
import { usePersistentTopN } from "../hooks/usePersistentTopN";
import SettingsContent, { MIN_TOP_N } from "../components/SettingsContent";
import { useMemo } from "react";

const ProjectChartWidget = () => {
  const [topN, setTopN] = usePersistentTopN("projectTopN", MIN_TOP_N);

  const options = useMemo(
    () => ({
      groupBy: "project" as Group,
      mode: "bar" as const,
      topN,
      collapseRemainder: true,
    }),
    [topN],
  );
  const { data, loading, keys } = useSummaryData(options);

  return (
    <WidgetCard
      title="Projects"
      loading={loading}
      settingsContent={
        <SettingsContent title="projects" topN={topN} setTopN={setTopN} />
      }
    >
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default ProjectChartWidget;
