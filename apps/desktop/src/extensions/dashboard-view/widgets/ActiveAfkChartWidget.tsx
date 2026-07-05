import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { addDays, format, startOfDay } from "date-fns";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "@/components/StackedBarChart";
import {
  BucketSummaryInput,
  BucketTimeSummary,
  commands,
  FullEvent,
} from "@/types/tauri.gen";
import { BarChartData } from "@/types/chart";

const ACTIVE_KEY = "Active";
const AFK_KEY = "AFK";
const KEYS = [ACTIVE_KEY, AFK_KEY];

type BucketWindow = {
  key: string;
  label: string;
  start: Date;
  end: Date;
};

const getDailyQuery = (start: Date, end: Date): BucketSummaryInput => ({
  preset: {
    custom: {
      start: start.toISOString(),
      end: end.toISOString(),
      bucket: "day",
    },
  },
});

const buildBucketWindows = (
  rangeStart: Date,
  rangeEnd: Date,
): BucketWindow[] => {
  const windows: BucketWindow[] = [];

  for (
    let cursor = startOfDay(rangeStart);
    cursor < rangeEnd;
    cursor = addDays(cursor, 1)
  ) {
    const end = addDays(cursor, 1);
    windows.push({
      key: format(cursor, "yyyy-MM-dd"),
      label: format(cursor, "MMM d"),
      start: cursor,
      end: end > rangeEnd ? rangeEnd : end,
    });
  }

  return windows;
};

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

  return buildBucketWindows(start, end).map((bucket) => {
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
      [ACTIVE_KEY]: activeByBucket.get(bucket.key) ?? 0,
      [AFK_KEY]: Math.round(afkSeconds),
    };
  });
};

type ActiveAfkChartWidgetProps = {
  startDate: Date;
  endDate: Date;
};

const ActiveAfkChartWidget = ({
  startDate,
  endDate,
}: ActiveAfkChartWidgetProps) => {
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
