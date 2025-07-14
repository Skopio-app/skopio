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

    const grouped: {
      date: Date;
      label: string;
      values: Record<string, number>;
    }[] = [];
    const allKeys = new Set<string>();
    const totalPerKey: Record<string, number> = {};

    for (const { bucket, grouped_values } of categoryBuckets) {
      const date = parseISO(bucket);
      const label = format(date, "MMM d");
      const values: Record<string, number> = {};

      for (const [category, seconds] of Object.entries(grouped_values)) {
        const value = seconds ?? 0;
        values[category] = seconds ?? 0;
        totalPerKey[category] = (totalPerKey[category] || 0) + value;
        allKeys.add(category);
      }

      grouped.push({ date, label, values });
    }

    grouped.sort((a, b) => a.date.getTime() - b.date.getTime());

    const sortedKeys = Array.from(allKeys).sort(
      (a, b) => (totalPerKey[b] ?? 0) - (totalPerKey[a] ?? 0),
    );

    const chartData: BarChartData[] = grouped.map(({ label, values }) => ({
      label,
      ...values,
    }));

    return [chartData, sortedKeys] as [BarChartData[], string[]];
  }, [rawGrouped.category]);

  return (
    <WidgetCard title="Categories" loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default CategoryChartWidget;
