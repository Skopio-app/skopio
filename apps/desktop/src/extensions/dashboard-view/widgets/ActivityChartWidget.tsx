import { useMemo } from "react";
import WidgetCard from "../components/WidgetCard";
import CalendarChart from "../charts/CalendarChart";
import { TimeRangePreset } from "../../../types/tauri.gen";
import { startOfYear } from "date-fns";
import { toNaiveDateTime } from "../dateRanges";
import { useSummaryData } from "../hooks/useSummaryData";
import { CalendarChartData } from "../types";

const ActivityChartWidget = () => {
  const preset = useMemo<TimeRangePreset>(() => {
    return {
      Custom: {
        start: toNaiveDateTime(startOfYear(new Date())),
        end: toNaiveDateTime(new Date()),
        bucket: "Day",
      },
    };
  }, []);

  const { getUngroupedData, loading } = useSummaryData(preset);

  const chartData = useMemo<CalendarChartData[]>(() => {
    return getUngroupedData().map(({ bucket, grouped_values }) => ({
      day: bucket,
      value: grouped_values["Total"] ?? 0,
    }));
  }, [getUngroupedData]);

  console.log("The chart data: ", chartData);

  return (
    <WidgetCard title="Daily Activity" loading={loading}>
      <CalendarChart data={chartData} />
    </WidgetCard>
  );
};

export default ActivityChartWidget;
