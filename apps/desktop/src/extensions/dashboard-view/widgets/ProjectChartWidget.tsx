import { useMemo } from "react";
import { BarChartData } from "../types";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "../charts/StackedBarChart";
import { useSummaryData } from "../hooks/useSummaryData";
import { format, parseISO } from "date-fns";

const ProjectChartWidget = () => {
  const { rawGrouped, loading } = useSummaryData(undefined, ["project"]);

  const [data, keys] = useMemo(() => {
    const projectBuckets = rawGrouped.project ?? [];

    const grouped: Record<string, Record<string, number>> = {};
    const allKeys = new Set<string>();

    for (const { bucket, grouped_values } of projectBuckets) {
      const label = format(parseISO(bucket), "MMM d");
      if (!grouped[label]) grouped[label] = {};

      for (const [project, seconds] of Object.entries(grouped_values)) {
        grouped[label][project] = seconds ?? 0;
        allKeys.add(project);
      }
    }

    const chartData: BarChartData[] = Object.entries(grouped).map(
      ([label, groupTotals]) => ({
        label,
        ...groupTotals,
      }),
    );

    return [chartData, Array.from(allKeys)] as [BarChartData[], string[]];
  }, [rawGrouped.project]);

  return (
    <WidgetCard title="Projects" loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default ProjectChartWidget;
