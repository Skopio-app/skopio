import { useEffect, useMemo, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
  Group,
} from "../../../types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";
import {
  BarChartData,
  LineChartData,
  PieChartData,
} from "../../../types/types";

export interface UseSummaryOptions {
  group_by?: Group;
  mode: "bar" | "line" | "list" | "pie";
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

type UseProjectSummaryDataFn = {
  (options: UseSummaryOptions & { mode: "line" }): ParsedLineChartResult;
  (options: UseSummaryOptions & { mode: "bar" }): ParsedBarChartResult;
  (options: UseSummaryOptions & { mode: "list" }): ParsedListResult;
  (options: UseSummaryOptions & { mode: "pie" }): ParsedPieChartResult;
};

// TODO: Parse entities to avoid having long strings.
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
    const date = new Date(bucket);
    const values: Record<string, number> = {};

    for (const [key, seconds] of Object.entries(grouped_values)) {
      const value = seconds ?? 0;
      values[key] = value;
      totalPerKey[key] = (totalPerKey[key] ?? 0) + value;
      allKeys.add(key);
    }

    grouped.push({ date, label: bucket, values });
  }

  grouped.sort((a, b) => a.date.getTime() - b.date.getTime());

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

const useProjectSummaryDataImpl = (
  rawOptions: UseSummaryOptions,
):
  | ParsedBarChartResult
  | ParsedLineChartResult
  | ParsedListResult
  | ParsedPieChartResult => {
  const options = useMemo(
    () => ({
      group_by: rawOptions.group_by,
      mode: rawOptions.mode,
    }),
    [rawOptions.group_by, rawOptions.mode],
  );

  const { preset, project } = usePresetFilter();
  const [loading, setLoading] = useState(true);
  const [rawData, setRawData] = useState<BucketTimeSummary[]>([]);

  useEffect(() => {
    const fetchData = async () => {
      setLoading(true);

      const query: BucketedSummaryInput = {
        preset,
        project_names: [project],
        group_by: options.group_by,
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
  }, [options.group_by, options.mode, project, preset]);

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
  }
};

export const useProjectSummaryData =
  useProjectSummaryDataImpl as UseProjectSummaryDataFn;
