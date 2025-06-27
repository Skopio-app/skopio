import {
  differenceInDays,
  endOfDay,
  endOfMonth,
  endOfWeek,
  format,
  startOfDay,
  startOfMonth,
  startOfWeek,
  subDays,
  subMonths,
  subWeeks,
} from "date-fns";
import { TimeBucket, TimeRangePreset } from "../../types/tauri.gen";

/**
 * Converts a JavaScript `Date` object to a naive datetime string in the format
 * `YYYY-MM-DDTHH:mm:ss`, which is compatible with Rust's `chrono::NaiveDateTime`.
 *
 * This format:
 * - Preserves the local date and time (does not convert to UTC)
 * - Omits timezone information
 * - Uses a `'T'` separator between date and time for JSON compatibility
 * - Is safe to pass to Tauri IPC commands and compatible with Serde parsing
 *
 * Example:
 * ```ts
 * const date = new Date(2025, 5, 24, 14, 30, 0); // June 24, 2025, 14:30:00
 * const naive = toNaiveDateTime(date);
 * console.log(naive); // "2025-06-24T14:30:00"
 * ```
 *
 * @param date - A valid JavaScript `Date` object
 * @returns A string in the format `"YYYY-MM-DDTHH:mm:ss"` representing the naive datetime
 */
export const toNaiveDateTime = (date: Date): string => {
  return format(date, "yyyy-MM-dd'T'HH:mm:ss");
};

export enum DateRangeType {
  Today = "Today",
  Yesterday = "Yesterday",
  Last7Days = "Last 7 days",
  Last14Days = "Last 14 days",
  Last30Days = "Last 30 days",
  ThisWeek = "This week",
  LastWeek = "Last week",
  ThisMonth = "This month",
  LastMonth = "Last month",
  Custom = "Custom range",
}

export const DATE_RANGE_LABELS = Object.values(DateRangeType);

export const getRangeDates = (
  range: DateRangeType,
  customStart?: Date,
  customEnd?: Date,
): [Date, Date] => {
  const today = startOfDay(new Date());

  switch (range) {
    case DateRangeType.Today:
      return [today, endOfDay(today)];
    case DateRangeType.Yesterday: {
      const yest = subDays(today, 1);
      return [yest, endOfDay(yest)];
    }
    case DateRangeType.Last7Days:
      return [subDays(today, 6), endOfDay(today)];
    case DateRangeType.Last14Days:
      return [subDays(today, 13), endOfDay(today)];
    case DateRangeType.Last30Days:
      return [subDays(today, 29), endOfDay(today)];
    case DateRangeType.ThisWeek:
      return [startOfWeek(today), endOfWeek(today)];
    case DateRangeType.LastWeek:
      return [startOfWeek(subWeeks(today, 1)), endOfWeek(subWeeks(today, 1))];
    case DateRangeType.ThisMonth:
      return [startOfMonth(today), endOfDay(today)];
    case DateRangeType.LastMonth: {
      const lastMonth = subMonths(today, 1);
      return [startOfMonth(lastMonth), endOfMonth(lastMonth)];
    }
    case DateRangeType.Custom:
      if (customStart && customEnd) return [customStart, customEnd];
      return [today, today];
  }
};

export const mapRangeToPreset = (
  range: DateRangeType,
  start: Date,
  end: Date,
): TimeRangePreset => {
  switch (range) {
    case DateRangeType.Today:
      return "Today";
    case DateRangeType.Yesterday:
      return "Yesterday";
    case DateRangeType.ThisWeek:
      return "ThisWeek";
    case DateRangeType.LastWeek:
      return "LastWeek";
    case DateRangeType.LastMonth:
      return "LastMonth";
    case DateRangeType.ThisMonth:
      return "ThisMonth";
    case DateRangeType.Last7Days:
      return { LastNDays: 7 };
    case DateRangeType.Last14Days:
      return { LastNDays: 14 };
    case DateRangeType.Last30Days:
      return { LastNDays: 30 };
    case DateRangeType.Custom: {
      const duration = differenceInDays(end, start);

      let bucket: TimeBucket;
      if (duration <= 1) {
        bucket = "Hour";
      } else if (duration <= 14) {
        bucket = "Day";
      } else if (duration <= 60) {
        bucket = "Week";
      } else {
        bucket = "Month";
      }

      return {
        Custom: {
          start: toNaiveDateTime(start),
          end: toNaiveDateTime(end),
          bucket,
        },
      };
    }
    default:
      return {
        Custom: {
          start: toNaiveDateTime(start),
          end: toNaiveDateTime(end),
          bucket: "Day",
        },
      };
  }
};

/**
 * Formats a duration given in seconds into a human-readable string.
 * @param seconds - The duration in seconds
 * @returns A string like "1h 45m 30s"
 */
export const formatDuration = (seconds: number): string => {
  const hrs = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  const padded = (n: number) => String(n).padStart(2, "0");
  const hrStr = `${hrs}h`;
  const minStr = `${padded(mins)}m`;
  const secStr = `${padded(secs)}s`;
  if (hrs > 0) {
    return `${hrStr} ${minStr} ${secStr}`;
  } else if (mins > 0) {
    return `${minStr} ${secStr}`;
  } else {
    return `${secStr}`;
  }
};

export const toHours = (seconds: number): number => {
  return parseFloat((seconds / 3600).toFixed(2));
};
