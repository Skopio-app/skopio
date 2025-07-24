import { useMemo } from "react";
import { BarChartData } from "../../../types/types";
import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import { format, parseISO } from "date-fns";
import StackedBarChart from "../../../components/StackedBarChart";

const ProjectChartWidget = () => {
  const { rawGrouped, loading } = useSummaryData(undefined, ["project"]);

  // TODO: Fix data display issue
  const [data, keys] = useMemo(() => {
    const projectBuckets = rawGrouped.project ?? [];

    const grouped: {
      date: Date;
      label: string;
      values: Record<string, number>;
    }[] = [];
    const allKeys = new Set<string>();
    const totalPerKey: Record<string, number> = {};

    for (const { bucket, grouped_values } of projectBuckets) {
      const parsedDate = parseISO(bucket);
      const label = format(parsedDate, "MMM d");
      const values: Record<string, number> = {};

      for (const [project, seconds] of Object.entries(grouped_values)) {
        const value = seconds ?? 0;
        values[project] = seconds ?? 0;
        totalPerKey[project] = (totalPerKey[project] || 0) + value;
        allKeys.add(project);
      }

      grouped.push({ date: parsedDate, label, values });
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
  }, [rawGrouped.project]);

  return (
    <WidgetCard title="Projects" loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default ProjectChartWidget;
