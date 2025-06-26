import { startTransition, useEffect, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
  Group,
  TimeRangePreset,
} from "../../../types/tauri.gen";
import { useDashboardFilter } from "../stores/useDashboardFilter";

interface UseSummaryResult {
  getGroupData: (group: Group) => Record<string, number>;
  getUngroupedData: () => BucketTimeSummary[];
  rawGrouped: Partial<Record<Group, BucketTimeSummary[]>>;
  ungrouped: BucketTimeSummary[] | null;
  loading: boolean;
  error: string | null;
}

export const useSummaryData = (
  overridePreset?: TimeRangePreset,
): UseSummaryResult => {
  const { preset: globalPreset } = useDashboardFilter();
  const preset = overridePreset ?? globalPreset;

  const [rawGrouped, setRawGrouped] = useState<
    Partial<Record<Group, BucketTimeSummary[]>>
  >({});
  const [ungrouped, setUngrouped] = useState<BucketTimeSummary[] | null>(null);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    const groupKeys: Group[] = [
      "app",
      "branch",
      "category",
      "entity",
      "language",
      "project",
    ];

    const run = async () => {
      setLoading(true);
      setError(null);

      if (overridePreset) {
        const input: BucketedSummaryInput = {
          preset: overridePreset,
          include_afk: false,
        };

        try {
          const result = await commands.fetchBucketedSummary(input);
          if (!Array.isArray(result) || cancelled) return;

          startTransition(() => {
            setUngrouped(result as BucketTimeSummary[]);
            setRawGrouped({});
            setLoading(false);
          });
        } catch (err) {
          setError((err as Error).message);
          setLoading(false);
        }

        return;
      }

      try {
        const results = await Promise.all(
          groupKeys.map(async (group_by) => {
            const input: BucketedSummaryInput = {
              preset,
              group_by,
              include_afk: false,
            };

            const result = await commands.fetchBucketedSummary(input);
            if (!Array.isArray(result)) return [group_by, []] as const;
            return [group_by, result] as const;
          }),
        );

        if (cancelled) return;

        const groupedData: Partial<Record<Group, BucketTimeSummary[]>> = {};
        for (const [key, result] of results) {
          groupedData[key] = result as BucketTimeSummary[];
        }

        startTransition(() => {
          setRawGrouped(groupedData);
          setUngrouped(null);
          setLoading(false);
        });
      } catch (err) {
        setError((err as Error).message);
        setLoading(false);
      }
    };

    run();

    return () => {
      cancelled = true;
    };
  }, [preset, overridePreset]);

  const getGroupData = (group: Group): Record<string, number> => {
    const data = rawGrouped[group];
    if (!data) return {};

    const totals: Record<string, number> = {};
    for (const { grouped_values } of data) {
      for (const [key, value] of Object.entries(grouped_values)) {
        totals[key] = (totals[key] ?? 0) + (value ?? 0);
      }
    }
    return totals;
  };

  const getUngroupedData = () => ungrouped ?? [];

  return {
    getGroupData,
    getUngroupedData,
    rawGrouped,
    ungrouped,
    loading,
    error,
  };
};
