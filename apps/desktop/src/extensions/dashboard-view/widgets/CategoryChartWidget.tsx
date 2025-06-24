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

const CategoryChartWidget = () => {
  const { preset } = useDashboardFilter();
  const [data, setData] = useState<BarChartData[]>([]);
  const [keys, setKeys] = useState<string[]>([]);

  useEffect(() => {
    const run = async () => {
      const input: BucketedSummaryInput = {
        preset,
        group_by: "activity_type",
        include_afk: false,
      };
      const result = await commands.fetchBucketedSummary(input);

      if (!Array.isArray(result)) return;

      const grouped: Record<string, Record<string, number>> = {};
      const allKeys = new Set<string>();

      for (const {
        group_key,
        bucket,
        total_seconds,
      } of result as BucketTimeSummary[]) {
        const label = format(parseISO(bucket), "MMM d");
        if (!grouped[label]) grouped[label] = {};
        grouped[label][group_key] = total_seconds;
        allKeys.add(group_key);
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
    <WidgetCard title="Categories" onRemove={() => {}}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default CategoryChartWidget;
