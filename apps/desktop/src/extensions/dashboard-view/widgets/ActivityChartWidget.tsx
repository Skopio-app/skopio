import { useEffect, useMemo, useState } from "react";
import WidgetCard from "../components/WidgetCard";
import CalendarChart from "../charts/CalendarChart";
import { TimeRangePreset } from "../../../types/tauri.gen";
import { endOfDay, endOfYear, startOfDay, startOfYear } from "date-fns";
import { toNaiveDateTime } from "../helpers/dateRanges";
import { useSummaryData } from "../hooks/useSummaryData";
import { CalendarChartData } from "../types";
import {
  getYearlyActivity,
  storeYearlyActivity,
  updateTodayActivity,
} from "../helpers/activityCache";
import { isDeltaOutdated } from "../helpers/delta";

const getTodayPreset = (): TimeRangePreset => ({
  Custom: {
    start: toNaiveDateTime(startOfDay(new Date())),
    end: toNaiveDateTime(endOfDay(new Date())),
    bucket: "Day",
  },
});

const getYearPreset = (): TimeRangePreset => ({
  Custom: {
    start: toNaiveDateTime(startOfYear(new Date())),
    end: toNaiveDateTime(endOfYear(new Date())),
    bucket: "Day",
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
      <CalendarChart data={data} />
    </WidgetCard>
  );
};

export default ActivityChartWidget;
