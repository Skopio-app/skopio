import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "@/components/StackedBarChart";
import { Group } from "@/types/tauri.gen";
import { useMemo } from "react";
import SettingsContent, { MIN_TOP_N } from "../components/SettingsContent";
import { useLocalStorage } from "@/hooks/useLocalStorage";

const EntityChartWidget = () => {
  const [topN, setTopN] = useLocalStorage("entityTopN", MIN_TOP_N);

  const options = useMemo(
    () => ({
      groupBy: "entity" as Group,
      mode: "bar" as const,
      topN,
      collapseRemainder: true,
    }),
    [topN],
  );
  const { loading, data, keys } = useSummaryData(options);

  return (
    <WidgetCard
      tooltip="The duration of use grouped by recorded file paths or URL segments."
      title="Entities"
      loading={loading}
      settingsContent={
        <SettingsContent title="entities" topN={topN} setTopN={setTopN} />
      }
    >
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default EntityChartWidget;
