import { BucketTimeSummary } from "../../../types/tauri.gen";
import { activityDB, CachedActivity } from "../db/activityDB";

export const storeYearlyActivity = async (summary: BucketTimeSummary[]) => {
  const now = Date.now();

  const items: CachedActivity[] = summary.map(({ bucket, grouped_values }) => ({
    day: bucket,
    year: new Date(bucket).getFullYear(),
    value: grouped_values["Total"] ?? 0,
    updated_at: now,
  }));

  await activityDB.activity.bulkPut(items);
};

export const getYearlyActivity = async (
  year: number,
): Promise<CachedActivity[]> => {
  return activityDB.activity.where("year").equals(year).toArray();
};

export const updateTodayActivity = async (
  todayBucket: BucketTimeSummary | null,
) => {
  if (!todayBucket) return;
  const value = todayBucket.grouped_values["Total"] ?? 0;
  const day = todayBucket.bucket;
  const year = new Date(day).getFullYear();

  await activityDB.activity.put({
    day,
    value,
    year,
    updated_at: Date.now(),
  });
};
