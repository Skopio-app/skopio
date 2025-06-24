import { useEffect, useState } from "react";
import StackedBarChart from "../charts/StackedBarChart";
import { useDashboardFilter } from "../stores/useDashboardFilter";
import WidgetCard from "../WidgetCard";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
} from "../../../types/tauri.gen";
import { format, parseISO } from "date-fns";

type BarChartData = {
  label: string;
  [project: string]: number | string;
};

const ProjectChartWidget = () => {
  const { preset } = useDashboardFilter();
  const [data, setData] = useState<BarChartData[]>([]);
  const [keys, setKeys] = useState<string[]>([]);

  useEffect(() => {
    const run = async () => {
      const input: BucketedSummaryInput = {
        preset: preset,
        group_by: "project",
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
    <WidgetCard title="Projects" onRemove={() => {}}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default ProjectChartWidget;
