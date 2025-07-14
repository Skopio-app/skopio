import { useEffect, useState, startTransition } from "react";
import { format, parseISO } from "date-fns";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "../charts/StackedBarChart";
import { useSummaryData } from "../hooks/useSummaryData";
import { BarChartData } from "../types";

const MAX_KEYS = 15;

const EntityChartWidget = () => {
  const { rawGrouped, loading } = useSummaryData(undefined, ["entity"]);
  const [chartData, setChartData] = useState<BarChartData[]>([]);
  const [topKeys, setTopKeys] = useState<string[]>([]);

  useEffect(() => {
    const entityBuckets = rawGrouped.entity ?? [];
    const totals: Record<string, number> = {};
    const grouped: {
      date: Date;
      label: string;
      values: Record<string, number>;
    }[] = [];
    const allKeys = new Set<string>();

    for (const { bucket, grouped_values } of entityBuckets) {
      const date = parseISO(bucket);
      const label = format(date, "MMM d");
      const values: Record<string, number> = {};

      for (const [entity, seconds] of Object.entries(grouped_values)) {
        const value = seconds ?? 0;
        values[entity] = seconds ?? 0;
        totals[entity] = (totals[entity] || 0) + value;
        allKeys.add(entity);
      }

      grouped.push({ date, label, values });
    }

    grouped.sort((a, b) => a.date.getTime() - b.date.getTime());

    const topEntities = Array.from(allKeys)
      .sort((a, b) => (totals[b] ?? 0) - (totals[a] ?? 0))
      .slice(0, MAX_KEYS);

    const finalData: BarChartData[] = grouped.map(({ label, values }) => ({
      label,
      ...values,
    }));

    startTransition(() => {
      setChartData(finalData);
      setTopKeys(topEntities);
    });
  }, [rawGrouped.entity]);

  return (
    <WidgetCard title="Entities" loading={loading}>
      <StackedBarChart data={chartData} keys={topKeys} />
    </WidgetCard>
  );
};

export default EntityChartWidget;
