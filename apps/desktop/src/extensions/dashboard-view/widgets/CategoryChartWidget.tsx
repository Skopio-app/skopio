import { useMemo } from "react";
import { BarChartData } from "../types";
import { format, parseISO } from "date-fns";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "../charts/StackedBarChart";
import { useSummaryData } from "../hooks/useSummaryData";

const CategoryChartWidget = () => {
  const { rawGrouped, loading } = useSummaryData(undefined, ["category"]);

  const [data, keys] = useMemo(() => {
    const categoryBuckets = rawGrouped.category ?? [];

    const grouped: Record<string, Record<string, number>> = {};
    const allKeys = new Set<string>();

    for (const { bucket, grouped_values } of categoryBuckets) {
      const label = format(parseISO(bucket), "MMM d");
      if (!grouped[label]) grouped[label] = {};

      for (const [category, seconds] of Object.entries(grouped_values)) {
        grouped[label][category] = seconds ?? 0;
        allKeys.add(category);
      }
    }

    const chartData: BarChartData[] = Object.entries(grouped).map(
      ([label, groupTotals]) => ({
        label,
        ...groupTotals,
      }),
    );

    return [chartData, Array.from(allKeys)] as [BarChartData[], string[]];
  }, [rawGrouped.category]);

  return (
    <WidgetCard title="Categories" loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default CategoryChartWidget;
