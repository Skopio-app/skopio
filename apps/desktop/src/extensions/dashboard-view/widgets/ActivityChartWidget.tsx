import { useMemo } from "react";
import WidgetCard from "../components/WidgetCard";
import { TimeRangePreset } from "@/types/tauri.gen";
import { endOfYear, startOfYear } from "date-fns";
import { useSummaryData } from "../hooks/useSummaryData";
import CalendarChart from "@/components/CalendarChart";

const getYearPreset = (): TimeRangePreset => ({
  custom: {
    start: startOfYear(new Date()).toISOString(),
    end: endOfYear(new Date()).toISOString(),
    bucket: "day",
  },
});

const ActivityChartWidget = () => {
  const yearPreset = useMemo(() => getYearPreset(), []);
  const options = { mode: "calendar" as const, presetOverride: yearPreset };
  const { loading, data } = useSummaryData(options);

  return (
    <WidgetCard
      tooltip="Daily recorded activity durations for the current year."
      title="Daily Activity"
      loading={loading}
      skeletonVariant="calendar"
    >
      <CalendarChart
        data={data}
        start={startOfYear(new Date())}
        end={new Date()}
      />
    </WidgetCard>
  );
};

export default ActivityChartWidget;
