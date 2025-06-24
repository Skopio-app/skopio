import { useEffect, useState } from "react";
import CustomPieChart from "../charts/CustomPieChart";
import { useDashboardFilter } from "../stores/useDashboardFilter";
import WidgetCard from "../components/WidgetCard";
import { BucketedSummaryInput, commands } from "../../../types/tauri.gen";
import { PieChartData } from "../types";

const AppPieChartWidget = () => {
  const { preset } = useDashboardFilter();
  const [data, setData] = useState<PieChartData[]>([]);

  useEffect(() => {
    const run = async () => {
      const input: BucketedSummaryInput = {
        preset,
        group_by: "app",
        include_afk: false,
      };
      const result = await commands.fetchBucketedSummary(input);

      if (!Array.isArray(result)) return;

      const totals: Record<string, number> = {};

      for (const { group_key, total_seconds } of result) {
        if (!totals[group_key]) totals[group_key] = 0;
        totals[group_key] += total_seconds;
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
    <WidgetCard title="Apps" onRemove={() => {}}>
      <CustomPieChart data={data} />
    </WidgetCard>
  );
};

export default AppPieChartWidget;
