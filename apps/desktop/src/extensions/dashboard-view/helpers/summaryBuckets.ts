import { format } from "date-fns";
import { TimeBucket, TimeRangePreset } from "@/types/tauri.gen";

export type DisplayBucket = "day" | "week" | "month" | "year";

export const toDisplayBucket = (bucket: TimeBucket): DisplayBucket =>
  bucket === "hour" ? "day" : bucket;

export const getDisplayBucketForPreset = (
  preset: TimeRangePreset,
): DisplayBucket => {
  if (typeof preset === "string") return "day";
  if ("custom" in preset) return toDisplayBucket(preset.custom.bucket);
  if ("lastNWeeks" in preset) return "week";
  if ("lastNMonths" in preset) return "month";
  if ("lastNYears" in preset) return "year";
  return "day";
};

const parseLocalDate = (key: string) => {
  const [year = "0", month = "1", day = "1"] = key.split("-");
  return new Date(Number(year), Number(month) - 1, Number(day));
};

const parseLocalMonth = (key: string) => {
  const [year = "0", month = "1"] = key.split("-");
  return new Date(Number(year), Number(month) - 1, 1);
};

export const getBucketLabel = (key: string, bucket: DisplayBucket) => {
  switch (bucket) {
    case "day":
      return format(parseLocalDate(key), "MMM d");
    case "month":
      return format(parseLocalMonth(key), "MMM yyyy");
    case "week":
    case "year":
      return key;
  }
};

export const getBucketOrder = (key: string, bucket: DisplayBucket) => {
  switch (bucket) {
    case "day":
      return parseLocalDate(key).getTime();
    case "month":
      return parseLocalMonth(key).getTime();
    case "year":
      return new Date(Number(key), 0, 1).getTime();
    case "week": {
      const [year = "0", week = "W00"] = key.split("-");
      return Number(year) * 100 + Number(week.replace("W", ""));
    }
  }
};

export const getSummaryBucketKey = (
  bucketKey: string,
  displayBucket: DisplayBucket,
) => {
  if (displayBucket === "day") return bucketKey.slice(0, 10);
  return bucketKey;
};
