import { useEffect, useState } from "react";
import WidgetCard from "../components/WidgetCard";
import CalendarChart from "../charts/CalendarChart";
import {
  BucketedSummaryInput,
  commands,
  TimeRangePreset,
} from "../../../types/tauri.gen";
import { startOfYear } from "date-fns";
import { toNaiveDateTime } from "../dateRanges";

type CalendarChartData = {
  value: number;
  day: string;
};

const ActivityChartWidget = () => {
  const [data, setData] = useState<CalendarChartData[]>([]);

  useEffect(() => {
    const run = async () => {
      const start = startOfYear(new Date());
      const end = new Date();
      const preset: TimeRangePreset = {
        Custom: {
          start: toNaiveDateTime(start),
          end: toNaiveDateTime(end),
          bucket: "Day",
        },
      };
      const input: BucketedSummaryInput = {
        preset,
        include_afk: false,
      };
      const result = await commands.fetchBucketedSummary(input);

      if (!Array.isArray(result)) return;

      const chartData: CalendarChartData[] = result.map(
        ({ bucket, total_seconds }) => ({
          day: bucket,
          value: total_seconds,
        }),
      );

      setData(chartData);
    };
    run();
  }, []);

  return (
    <WidgetCard title="Daily Activity" onRemove={() => {}}>
      <CalendarChart data={data} />
    </WidgetCard>
  );
};

export default ActivityChartWidget;
