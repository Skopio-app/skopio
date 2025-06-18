import {
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
