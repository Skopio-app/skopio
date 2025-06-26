import { useMemo } from "react";
import { BarChartData } from "../types";
import { format, parseISO } from "date-fns";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "../charts/StackedBarChart";
import { useSummaryData } from "../hooks/useSummaryData";

const EntityChartWidget = () => {
  const { rawGrouped, loading } = useSummaryData();

  const [data, keys] = useMemo(() => {
    const entityBuckets = rawGrouped.entity ?? [];

    const grouped: Record<string, Record<string, number>> = {};
    const allKeys = new Set<string>();

    for (const { bucket, grouped_values } of entityBuckets) {
      const label = format(parseISO(bucket), "MMM d");
      if (!grouped[label]) grouped[label] = {};

      for (const [entity, seconds] of Object.entries(grouped_values)) {
        grouped[label][entity] = seconds ?? 0;
        allKeys.add(entity);
      }
    }

    const chartData: BarChartData[] = Object.entries(grouped).map(
      ([label, groupTotals]) => ({
        label,
        ...groupTotals,
      }),
    );

    return [chartData, Array.from(allKeys)] as [BarChartData[], string[]];
  }, [rawGrouped.entity]);

  return (
    <WidgetCard title="Entities" loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default EntityChartWidget;
