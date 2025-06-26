import { useEffect, useState } from "react";
import { useDashboardFilter } from "../stores/useDashboardFilter";
import { BarChartData } from "../types";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
} from "../../../types/tauri.gen";
import { format, parseISO } from "date-fns";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "../charts/StackedBarChart";

const EntityChartWidget = () => {
  const { preset } = useDashboardFilter();
  const [data, setData] = useState<BarChartData[]>([]);
  const [keys, setKeys] = useState<string[]>([]);

  useEffect(() => {
    const run = async () => {
      const input: BucketedSummaryInput = {
        preset,
        group_by: "entity",
        include_afk: false,
      };
      const result = await commands.fetchBucketedSummary(input);

      if (!Array.isArray(result)) return;

      const grouped: Record<string, Record<string, number>> = {};
      const allKeys = new Set<string>();

      for (const { bucket, grouped_values } of result as BucketTimeSummary[]) {
        const label = format(parseISO(bucket), "MMM d");
        if (!grouped[label]) grouped[label] = {};

        for (const [key, value] of Object.entries(grouped_values)) {
          grouped[label][key] = value ?? 0;
          allKeys.add(key);
        }
      }

      const chartData: BarChartData[] = Object.entries(grouped).map(
        ([label, groupTotals]) => ({
          label,
          ...groupTotals,
        }),
      );
      setData(chartData);
      setKeys(Array.from(allKeys));
    };

    run();
  }, [preset]);

  return (
    <WidgetCard title="Entities" onRemove={() => {}}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default EntityChartWidget;
