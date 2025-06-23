import { useEffect, useState } from "react";
import AppPieChart from "../charts/AppPieChart";
import { useDashboardFilter } from "../stores/useDashboardFilter";
import WidgetCard from "../WidgetCard";
import { BucketedSummaryInput, commands } from "../../../types/tauri.gen";

type PieChartData = {
  id: string;
  label: string;
  value: number;
};

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
      console.log("The result: ", result);

      if (!Array.isArray(result)) return;

      const totals: Record<string, number> = {};

      for (const { group_key, total_seconds } of result) {
        if (!totals[group_key]) totals[group_key] = 0;
        totals[group_key] += total_seconds;
      }

      console.log("totals: ", totals);
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
      <AppPieChart data={data} />
    </WidgetCard>
  );
};

export default AppPieChartWidget;
