import { BucketTimeSummary } from "../../../types/tauri.gen";
import { CalendarChartData } from "../types";

export const isDeltaOutdated = (
  cached: CalendarChartData | undefined,
  incoming: BucketTimeSummary,
  thresholdSeconds = 60,
): { outdated: boolean; delta: number } => {
  const newValue = incoming.grouped_values["Total"] ?? 0;
  const oldValue = cached?.value ?? 0;

  const delta = Math.abs(newValue - oldValue);
  const outdated = delta > thresholdSeconds;

  return { outdated, delta };
};
