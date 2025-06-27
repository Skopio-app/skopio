import { BucketTimeSummary } from "../../../types/tauri.gen";
import { CachedActivity, skopioDB } from "../db/skopioDB";

export const storeYearlyActivity = async (summary: BucketTimeSummary[]) => {
  const now = Date.now();
  const year = new Date(summary[0].bucket).getFullYear();

  const values = summary.map(({ bucket, grouped_values }) => ({
    day: bucket,
    value: grouped_values["Total"] ?? 0,
  }));

  const group: CachedActivity = {
    year,
    values,
    updated_at: now,
  };

  await skopioDB.activity.put(group);
};

export const getYearlyActivity = async (
  year: number,
): Promise<CachedActivity | null> => {
  const result = await skopioDB.activity.get(year);
  return result ?? null;
};

export const updateTodayActivity = async (
  todayBucket: BucketTimeSummary | null,
) => {
  if (!todayBucket) return;
  const value = todayBucket.grouped_values["Total"] ?? 0;
  const day = todayBucket.bucket;
  const year = new Date(day).getFullYear();

  const existing = await skopioDB.activity.get(year);

  const updated = {
    year,
    updated_at: Date.now(),
    values: (existing?.values ?? [])
      .filter((d) => d.day !== day)
      .concat({ day, value }),
  };

  updated.values.sort(
    (a, b) => new Date(a.day).getTime() - new Date(b.day).getTime(),
  );

  await skopioDB.activity.put(updated);
};
