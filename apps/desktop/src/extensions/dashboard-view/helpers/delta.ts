import { BucketTimeSummary } from "../../../types/tauri.gen";
import { CachedActivity } from "../db/activityDB";

export const isDeltaOutdated = (
  cached: CachedActivity | undefined,
  incoming: BucketTimeSummary,
  thresholdSeconds = 60,
): { outdated: boolean; delta: number } => {
  const newValue = incoming.grouped_values["Total"] ?? 0;
  const oldValue = cached?.value ?? 0;

  const delta = Math.abs(newValue - oldValue);
  const outdated = delta > thresholdSeconds;

  return { outdated, delta };
};
