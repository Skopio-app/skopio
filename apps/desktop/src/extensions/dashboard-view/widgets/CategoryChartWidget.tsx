import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import StackedBarChart from "../../../components/StackedBarChart";
import { Group } from "../../../types/tauri.gen";
import { useMemo, useState } from "react";
import { Slider } from "@skopio/ui";

const MIN_TOP_N = 2;
const MAX_TOP_N = 20;

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

  const settingsContent = (
    <div className="w-64">
      <div className="mb-2 text-sm font-medium text-muted-foreground">
        Show top N categories
      </div>
      <div className="flex items-center gap-3">
        <Slider
          value={[topN]}
          min={MIN_TOP_N}
          max={MAX_TOP_N}
          step={1}
          onValueChange={(v) => setTopN(v[0] ?? MIN_TOP_N)}
          aria-label="Top N"
          className="flex-1"
        />
        <p className="w-10 text-right text-sm tabular-nums">{topN}</p>
      </div>
      <p className="mt-2 text-xs text-muted-foreground">
        Only the top {topN} series are shown. The rest are grouped into "Other".
      </p>
    </div>
  );

  return (
    <WidgetCard
      title="Categories"
      loading={loading}
      settingsContent={settingsContent}
    >
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default CategoryChartWidget;
