import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "../../../components/StackedBarChart";
import { Group } from "../../../types/tauri.gen";
import { useMemo, useState } from "react";
import SettingsContent, { MIN_TOP_N } from "../components/SettingsContent";

const CategoryChartWidget = () => {
  const [topN, setTopN] = useState<number>(MIN_TOP_N);
  const options = useMemo(
    () => ({
      groupBy: "category" as Group,
      mode: "bar" as const,
      topN,
      collapseRemainder: true,
    }),
    [topN],
  );
  const { loading, data, keys } = useSummaryData(options);

  return (
    <WidgetCard
      title="Categories"
      loading={loading}
      settingsContent={
        <SettingsContent
          title="Categories"
          min={MIN_TOP_N}
          topN={topN}
          setTopN={setTopN}
        />
      }
    >
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default CategoryChartWidget;
