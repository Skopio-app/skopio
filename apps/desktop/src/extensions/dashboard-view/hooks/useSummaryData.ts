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
  const byLabel: Map<string, Record<string, number>> = new Map();
  const labelToDate: Map<string, Date> = new Map();

  const allKeys = new Set<string>();
  const totalPerKey: Record<string, number> = {};

  for (const { bucket, grouped_values } of rawData) {
    const date = parseISO(bucket);
    const label = format(date, "MMM d");

    if (!byLabel.has(label)) {
      byLabel.set(label, {});
      labelToDate.set(label, date);
      console.log("byLabel: ", byLabel);
      console.log("labelToDate: ", labelToDate);
    }

    const acc = byLabel.get(label)!;

    for (const [rawKey, seconds] of Object.entries(grouped_values)) {
      if (rawKey === "Total") continue;
      const key = String(rawKey);
      const value = Number(seconds ?? 0);

      if (value <= 0) continue;

      acc[key] = (acc[key] ?? 0) + value;
      totalPerKey[key] = (totalPerKey[key] ?? 0) + value;
      allKeys.add(key);
    }
  }

  const sortedKeys = Array.from(allKeys).sort(
    (a, b) => (totalPerKey[b] ?? 0) - (totalPerKey[a] ?? 0),
  );

  const chartData: BarChartData[] = Array.from(byLabel.entries())
    .map(([label, values]) => {
      const row: Record<string, number | string> = { label };
      for (const k of sortedKeys) {
        row[k] = values[k] ?? 0;
      }
      return row as BarChartData;
    })
    .sort(
      (a, b) =>
        labelToDate.get(a.label)!.getTime() -
        labelToDate.get(b.label)!.getTime(),
    );

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
