import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "@/components/StackedBarChart";
import {
  BucketSummaryInput,
  BucketTimeSummary,
  commands,
  FullEvent,
} from "@/types/tauri.gen";
import { BarChartData } from "@/types/chart";
import { getDailyBucketWindows, mapDatesToDailyPreset } from "@/utils/time";
import { useDashboardFilter } from "../stores/useDashboardFilter";

const ACTIVE_KEY = "Active";
const AFK_KEY = "AFK";
const KEYS = [ACTIVE_KEY, AFK_KEY];

const getDailyQuery = (start: Date, end: Date): BucketSummaryInput => ({
  preset: mapDatesToDailyPreset(start, end),
});

const getAfkEventEnd = (event: FullEvent) => {
  if (event.endTimestamp) return new Date(event.endTimestamp);
  if (event.duration) {
    return new Date(
      new Date(event.timestamp).getTime() + event.duration * 1000,
    );
  }
  return new Date();
};

const buildActiveAfkData = (
  start: Date,
  end: Date,
  activeBuckets: BucketTimeSummary[],
  afkEvents: FullEvent[],
): BarChartData[] => {
  const activeByBucket = new Map(
    activeBuckets.map((bucket) => [
      bucket.bucket,
      bucket.groupedValues.Total ?? 0,
    ]),
  );
  const bucketWindows = getDailyBucketWindows(start, end);
  const omitEmptyDays = bucketWindows.length > 1;

  return bucketWindows
    .map((bucket) => {
      const activeSeconds = activeByBucket.get(bucket.key) ?? 0;
      const afkSeconds = afkEvents.reduce((total, event) => {
        const eventStart = new Date(event.timestamp);
        const eventEnd = getAfkEventEnd(event);
        const overlapStart = Math.max(
          eventStart.getTime(),
          bucket.start.getTime(),
        );
        const overlapEnd = Math.min(eventEnd.getTime(), bucket.end.getTime());

        if (overlapEnd <= overlapStart) return total;

        return total + (overlapEnd - overlapStart) / 1000;
      }, 0);

      return {
        label: bucket.label,
        [ACTIVE_KEY]: activeSeconds,
        [AFK_KEY]: Math.round(afkSeconds),
      };
    })
    .filter(
      (row) =>
        !omitEmptyDays ||
        Number(row[ACTIVE_KEY] ?? 0) > 0 ||
        Number(row[AFK_KEY] ?? 0) > 0,
    );
};

const ActiveAfkChartWidget = () => {
  const { startDate, endDate } = useDashboardFilter();
  const query = useMemo(
    () => getDailyQuery(startDate, endDate),
    [startDate, endDate],
  );
  const rangeKey = useMemo(
    () => ({
      start: startDate.toISOString(),
      end: endDate.toISOString(),
    }),
    [startDate, endDate],
  );

  const { data, isLoading } = useQuery({
    queryKey: ["dashboardActiveAfk", rangeKey],
    queryFn: async () => {
      const [activeBuckets, afkEvents] = await Promise.all([
        commands.fetchBucketedSummary(query),
        commands.fetchAfkEvents(query),
      ]);

      return buildActiveAfkData(startDate, endDate, activeBuckets, afkEvents);
    },
  });

  return (
    <WidgetCard
      tooltip="Tracked active time compared with detected AFK time for the selected date range."
      title="Active vs AFK"
      loading={isLoading}
    >
      <StackedBarChart data={data ?? []} keys={KEYS} bucketLabel="Time" />
    </WidgetCard>
  );
};

export default ActiveAfkChartWidget;
