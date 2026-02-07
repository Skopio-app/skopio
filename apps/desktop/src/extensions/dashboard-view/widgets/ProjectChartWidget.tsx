import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "@/components/StackedBarChart";
import { Group } from "@/types/tauri.gen";
import SettingsContent, { MIN_TOP_N } from "../components/SettingsContent";
import { useMemo } from "react";
import { useLocalStorage } from "@/hooks/useLocalStorage";

const ProjectChartWidget = () => {
  const [topN, setTopN] = useLocalStorage("projectTopN", MIN_TOP_N);

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
      tooltip="The duration of use of the various recorded domains, app names, and projects (shown by their folder names)."
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
