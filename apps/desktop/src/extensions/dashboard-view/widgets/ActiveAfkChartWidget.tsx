import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import WidgetCard from "../components/WidgetCard";
import StackedBarChart from "@/components/StackedBarChart";
import { BucketTimeSummary, commands } from "@/types/tauri.gen";
import { BarChartData } from "@/types/chart";
import {
  DisplayBucket,
  getBucketLabel,
  getBucketOrder,
  getDisplayBucketForPreset,
  getSummaryBucketKey,
} from "../helpers/summaryBuckets";
import { useSummaryBuckets } from "../hooks/useSummaryData";

const ACTIVE_KEY = "Active";
const AFK_KEY = "AFK";
const KEYS = [ACTIVE_KEY, AFK_KEY];

type ChartRow = BarChartData & {
  label: string;
  [ACTIVE_KEY]: number;
  [AFK_KEY]: number;
};

const buildActiveAfkData = (
  displayBucket: DisplayBucket,
  activeBuckets: BucketTimeSummary[],
  afkBuckets: BucketTimeSummary[],
): BarChartData[] => {
  const rows = new Map<string, { row: ChartRow; order: number }>();

  const ensureRow = (key: string) => {
    const existing = rows.get(key);
    if (existing) return existing.row;

    const row: ChartRow = {
      label: getBucketLabel(key, displayBucket),
      [ACTIVE_KEY]: 0,
      [AFK_KEY]: 0,
    };
    rows.set(key, { row, order: getBucketOrder(key, displayBucket) });
    return row;
  };

  for (const bucket of activeBuckets) {
    const key = getSummaryBucketKey(bucket.bucket, displayBucket);
    const row = ensureRow(key);
    row[ACTIVE_KEY] += bucket.groupedValues.Total ?? 0;
  }

  for (const bucket of afkBuckets) {
    const key = getSummaryBucketKey(bucket.bucket, displayBucket);
    const row = ensureRow(key);
    row[AFK_KEY] += bucket.groupedValues.Total ?? 0;
  }

  return Array.from(rows.values())
    .filter(
      ({ row }) =>
        Number(row[ACTIVE_KEY] ?? 0) > 0 || Number(row[AFK_KEY] ?? 0) > 0,
    )
    .sort((a, b) => a.order - b.order)
    .map(({ row }) => row);
};

const ActiveAfkChartWidget = () => {
  const {
    data: activeBuckets,
    loading: activeLoading,
    preset,
  } = useSummaryBuckets();
  const displayBucket = useMemo(
    () => getDisplayBucketForPreset(preset),
    [preset],
  );

  const { data: afkBuckets = [], isLoading: afkLoading } = useQuery({
    queryKey: ["dashboardActiveAfkBuckets", preset],
    queryFn: () => commands.fetchBucketedAfkSummary({ preset }),
  });

  const data = useMemo(
    () => buildActiveAfkData(displayBucket, activeBuckets, afkBuckets),
    [activeBuckets, afkBuckets, displayBucket],
  );

  return (
    <WidgetCard
      tooltip="Tracked active time compared with detected AFK time for the selected date range."
      title="Active vs AFK"
      loading={activeLoading || afkLoading}
    >
      <StackedBarChart data={data ?? []} keys={KEYS} bucketLabel="Days" />
    </WidgetCard>
  );
};

export default ActiveAfkChartWidget;
