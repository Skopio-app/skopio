import { useEffect, useState, startTransition } from "react";
import { format, parseISO } from "date-fns";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "../charts/StackedBarChart";
import { useSummaryData } from "../hooks/useSummaryData";
import { BarChartData } from "../types";

const MAX_KEYS = 20;

const EntityChartWidget = () => {
  const { rawGrouped, loading } = useSummaryData(undefined, ["entity"]);
  const [chartData, setChartData] = useState<BarChartData[]>([]);
  const [topKeys, setTopKeys] = useState<string[]>([]);

  useEffect(() => {
    const entityBuckets = rawGrouped.entity ?? [];
    const totals: Record<string, number> = {};
    const grouped: Record<string, Record<string, number>> = {};
    const groupedBuckets: Record<string, string> = {};

    for (const { bucket, grouped_values } of entityBuckets) {
      const date = parseISO(bucket);
      const label = format(date, "MMM d");
      groupedBuckets[label] = bucket;

      if (!grouped[label]) grouped[label] = {};

      for (const [entity, seconds] of Object.entries(grouped_values)) {
        grouped[label][entity] = seconds ?? 0;
        totals[entity] = (totals[entity] ?? 0) + (seconds ?? 0);
      }
    }

    const topEntities = Object.entries(totals)
      .sort((a, b) => b[1] - a[1])
      .slice(0, MAX_KEYS)
      .map(([key]) => key);

    const parsedGrouped = Object.entries(grouped).map(
      ([label, groupTotals]) => {
        const date = parseISO(groupedBuckets[label]);
        return { date, label, groupTotals };
      },
    );

    parsedGrouped.sort((a, b) => a.date.getTime() - b.date.getTime());

    const finalData: BarChartData[] = parsedGrouped.map(
      ({ label, groupTotals }) => {
        const entry: BarChartData = { label };
        for (const key of topEntities) {
          entry[key] = groupTotals[key] ?? 0;
        }
        return entry;
      },
    );

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
