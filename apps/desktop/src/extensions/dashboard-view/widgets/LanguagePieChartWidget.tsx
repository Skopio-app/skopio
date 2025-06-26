import { useEffect, useState } from "react";
import { useDashboardFilter } from "../stores/useDashboardFilter";
import { PieChartData } from "../types";
import { BucketedSummaryInput, commands } from "../../../types/tauri.gen";
import WidgetCard from "../components/WidgetCard";
import CustomPieChart from "../charts/CustomPieChart";

const LanguagePieChartWidget = () => {
  const { preset } = useDashboardFilter();
  const [data, setData] = useState<PieChartData[]>([]);

  useEffect(() => {
    const run = async () => {
      const input: BucketedSummaryInput = {
        preset,
        group_by: "language",
        include_afk: false,
      };
      const result = await commands.fetchBucketedSummary(input);

      if (!Array.isArray(result)) return;

      const totals: Record<string, number> = {};

      for (const { grouped_values } of result) {
        for (const [key, value] of Object.entries(grouped_values)) {
          if (!totals[key]) totals[key] = 0;
          totals[key] += value ?? 0;
        }
      }

      const chartData: PieChartData[] = Object.entries(totals).map(
        ([app, seconds]) => ({
          id: app,
          label: app,
          value: seconds,
        }),
      );
      setData(chartData);
    };

    run();
  }, [preset]);

  return (
    <WidgetCard title="Languages" onRemove={() => {}}>
      <CustomPieChart data={data} />
    </WidgetCard>
  );
};

export default LanguagePieChartWidget;
