import { useEffect, useMemo, useState } from "react";
import WidgetCard from "../components/WidgetCard";
import { TimeRangePreset } from "../../../types/tauri.gen";
import { endOfYear, startOfYear } from "date-fns";
import { useSummaryData } from "../hooks/useSummaryData";
import { CalendarChartData } from "../../../types/types";
import {
  getYearlyActivity,
  storeYearlyActivity,
  updateTodayActivity,
} from "../helpers/activityCache";
import { isDeltaOutdated } from "../helpers/delta";
import CalendarChart from "../../../components/CalendarChart";

const getTodayPreset = (): TimeRangePreset => "today";

const getYearPreset = (): TimeRangePreset => ({
  custom: {
    start: startOfYear(new Date()).toISOString(),
    end: endOfYear(new Date()).toISOString(),
    bucket: "day",
  },
});

const ActivityChartWidget = () => {
  const currentYear = new Date().getFullYear();
  const yearPreset = useMemo(() => getYearPreset(), []);
  const todayPreset = useMemo(() => getTodayPreset(), []);

  const [data, setData] = useState<CalendarChartData[]>([]);

  const { getUngroupedData: getYearData, loading: yearLoading } =
    useSummaryData(yearPreset);
  const { getUngroupedData: getTodayData, loading: todayLoading } =
    useSummaryData(todayPreset);

  useEffect(() => {
    let cancelled = false;

    const loadAndReconcile = async () => {
      let cached = await getYearlyActivity(currentYear);
      if (cancelled) return;

      if (!cached || cached.values.length === 0) {
        const fresh = getYearData();
        const values = fresh.map(({ bucket, grouped_values }) => ({
          day: bucket,
          value: grouped_values["Total"] ?? 0,
          year: new Date(bucket).getFullYear(),
          updated_at: Date.now(),
        }));

        await storeYearlyActivity(fresh);
        cached = { year: currentYear, values, updated_at: Date.now() };
      }
      if (cached) {
        setData(cached.values);
      }

      // TODO: Fix activity chart not updating total time
      const [todayBucket] = getTodayData();
      if (!todayBucket) return;

      const cachedToday = cached.values.find(
        (c) => c.day === todayBucket.bucket,
      );
      const { outdated } = isDeltaOutdated(cachedToday, todayBucket);

      if (outdated) {
        await updateTodayActivity(todayBucket);

        const updated = cached.values
          .filter((c) => c.day !== todayBucket.bucket)
          .concat({
            day: todayBucket.bucket,
            value: todayBucket.grouped_values["Total"] ?? 0,
          });

        updated.sort(
          (a, b) => new Date(a.day).getTime() - new Date(b.day).getTime(),
        );

        if (!cancelled) {
          setData(updated);
        }
      }
    };

    loadAndReconcile();

    return () => {
      cancelled = true;
    };
  }, [currentYear, getTodayData]);

  return (
    <WidgetCard title="Daily Activity" loading={todayLoading || yearLoading}>
      <CalendarChart
        data={data}
        start={startOfYear(new Date())}
        end={new Date()}
      />
    </WidgetCard>
  );
};

export default ActivityChartWidget;
