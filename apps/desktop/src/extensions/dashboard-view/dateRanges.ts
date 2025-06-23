import {
  differenceInDays,
  endOfDay,
  endOfMonth,
  endOfWeek,
  startOfDay,
  startOfMonth,
  startOfWeek,
  subDays,
  subMonths,
  subWeeks,
} from "date-fns";
import { TimeBucket, TimeRangePreset } from "../../types/tauri.gen";

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
          start: start.toISOString(),
          end: end.toISOString(),
          bucket,
        },
      };
    }
    default:
      return {
        Custom: {
          start: start.toISOString(),
          end: end.toISOString(),
          bucket: "Day",
        },
      };
  }
};

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
