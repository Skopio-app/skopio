import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "../../../components/StackedBarChart";
import { Group } from "../../../types/tauri.gen";
import { useMemo } from "react";
import { usePersistentTopN } from "../hooks/usePersistentTopN";
import SettingsContent, { MIN_TOP_N } from "../components/SettingsContent";

const EntityChartWidget = () => {
  const [topN, setTopN] = usePersistentTopN("entityTopN", MIN_TOP_N);

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
