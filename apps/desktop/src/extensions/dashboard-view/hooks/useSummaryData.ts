import { useEffect, useMemo, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
  Group,
  TimeRangePreset,
} from "../../../types/tauri.gen";
import {
  BarChartData,
  CalendarChartData,
  LineChartData,
  PieChartData,
} from "../../../types/types";
import { useDashboardFilter } from "../stores/useDashboardFilter";
import { format, parseISO } from "date-fns";

export interface UseSummaryOptions {
  groupBy?: Group;
  mode: "bar" | "line" | "list" | "pie" | "calendar";
  presetOverride?: TimeRangePreset;
}

interface LoadingResult {
  loading: boolean;
}

interface ParsedBarChartResult extends LoadingResult {
  data: BarChartData[];
  keys: string[];
}

interface ParsedLineChartResult extends LoadingResult {
  data: LineChartData[];
}

interface ParsedListResult extends LoadingResult {
  data: { name: string; value: number }[];
}

interface ParsedPieChartResult extends LoadingResult {
  data: PieChartData[];
}
interface ParsedCalendarChartResult extends LoadingResult {
  data: CalendarChartData[];
}

type UseSummaryDataFn = {
  (options: UseSummaryOptions & { mode: "line" }): ParsedLineChartResult;
  (options: UseSummaryOptions & { mode: "bar" }): ParsedBarChartResult;
  (options: UseSummaryOptions & { mode: "list" }): ParsedListResult;
  (options: UseSummaryOptions & { mode: "pie" }): ParsedPieChartResult;
  (
    options: UseSummaryOptions & { mode: "calendar" },
  ): ParsedCalendarChartResult;
};

// Shared logic for bar chart grouping
const generateGroupedChartData = (rawData: BucketTimeSummary[]) => {
  const grouped: {
    date: Date;
    label: string;
    values: Record<string, number>;
  }[] = [];
  const allKeys = new Set<string>();
  const totalPerKey: Record<string, number> = {};

  for (const { bucket, grouped_values } of rawData) {
    const date = parseISO(bucket);
    const label = format(date, "MMM d");
    const values: Record<string, number> = {};

    for (const [key, seconds] of Object.entries(grouped_values)) {
      const value = seconds ?? 0;
      values[key] = value;
      totalPerKey[key] = (totalPerKey[key] ?? 0) + value;
      allKeys.add(key);
    }

    grouped.push({ date, label, values });
  }

  grouped.sort((a, b) => a.date.getTime() - b.date.getTime());
  console.log("Total per key: ", totalPerKey);

  const sortedKeys = Array.from(allKeys).sort(
    (a, b) => (totalPerKey[b] ?? 0) - (totalPerKey[a] ?? 0),
  );

  const chartData: BarChartData[] = grouped.map(({ label, values }) => ({
    label,
    ...values,
  }));

  return { chartData, sortedKeys };
};

// Shared logic to merge values
const mergeGroupedValues = (data: BucketTimeSummary[]) => {
  const merged: Record<string, number> = {};
  for (const item of data) {
    for (const [key, value] of Object.entries(item.grouped_values)) {
      merged[key] = (merged[key] ?? 0) + (value ?? 0);
    }
  }

  return merged;
};

const useSummaryDataImpl = (
  rawOptions: UseSummaryOptions,
):
  | ParsedBarChartResult
  | ParsedLineChartResult
  | ParsedListResult
  | ParsedPieChartResult
  | ParsedCalendarChartResult => {
  const options = useMemo(
    () => ({
      groupBy: rawOptions.groupBy,
      mode: rawOptions.mode,
      presetOverride: rawOptions.presetOverride,
    }),
    [rawOptions.groupBy, rawOptions.mode, rawOptions.presetOverride],
  );

  const { preset: dashboardPreset } = useDashboardFilter();
  const [loading, setLoading] = useState(true);
  const [rawData, setRawData] = useState<BucketTimeSummary[]>([]);
  const preset = options.presetOverride ?? dashboardPreset;

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);

      const query: BucketedSummaryInput = {
        preset,
        group_by: options.presetOverride ? null : options.groupBy,
        include_afk: false,
      };

      try {
        const summary = await commands.fetchBucketedSummary(query);
        setRawData(summary);
      } catch (err) {
        console.error("Error fetching summary data: ", err);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [options.groupBy, options.mode, JSON.stringify(preset)]);

  switch (options.mode) {
    case "line": {
      const data: LineChartData[] = rawData.map((item) => ({
        x: item.bucket,
        y: item.grouped_values["Total"] ?? 0,
      }));
      return { data, loading };
    }

    case "bar": {
      const { chartData, sortedKeys } = generateGroupedChartData(rawData);
      return { data: chartData, keys: sortedKeys, loading };
    }

    case "pie": {
      const merged = mergeGroupedValues(rawData);
      const data: PieChartData[] = Object.entries(merged).map(
        ([id, value]) => ({
          id,
          label: id,
          value,
        }),
      );
      return { data, loading };
    }

    case "list": {
      const merged = mergeGroupedValues(rawData);
      const data = Object.entries(merged)
        .map(([name, value]) => ({ name, value }))
        .sort((a, b) => b.value - a.value);
      return { data, loading };
    }

    case "calendar": {
      const data: CalendarChartData[] = rawData.map(
        ({ bucket, grouped_values }) => ({
          day: bucket,
          value: grouped_values["Total"] ?? 0,
        }),
      );

      const sorted = [...data].sort(
        (a, b) => new Date(a.day).getTime() - new Date(b.day).getTime(),
      );

      return { data: sorted, loading };
    }
  }
};

export const useSummaryData = useSummaryDataImpl as UseSummaryDataFn;
