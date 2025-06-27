import { startTransition, useEffect, useMemo, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
  Group,
  TimeRangePreset,
} from "../../../types/tauri.gen";
import { useDashboardFilter } from "../stores/useDashboardFilter";
import pMap from "p-map";

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
  groups: Group[] = [],
): UseSummaryResult => {
  const { preset: globalPreset } = useDashboardFilter();
  const preset = overridePreset ?? globalPreset;

  const [rawGrouped, setRawGrouped] = useState<
    Partial<Record<Group, BucketTimeSummary[]>>
  >({});
  const [ungrouped, setUngrouped] = useState<BucketTimeSummary[] | null>(null);

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const groupKeys = useMemo(() => JSON.stringify([...groups].sort()), [groups]);

  useEffect(() => {
    let cancelled = false;

    const run = async () => {
      setLoading(true);
      setError(null);

      if (overridePreset || groups.length === 0) {
        const input: BucketedSummaryInput = {
          preset,
          include_afk: false,
        };

        try {
          const result = await commands.fetchBucketedSummary(input);
          if (!Array.isArray(result) || cancelled) return;

          startTransition(() => {
            setUngrouped(result);
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
        startTransition(() => {
          setRawGrouped({});
          setUngrouped(null);
        });

        await pMap(
          groups,
          async (group_by) => {
            const input: BucketedSummaryInput = {
              preset,
              group_by,
              include_afk: false,
            };

            const result = await commands.fetchBucketedSummary(input);
            if (!Array.isArray(result) || cancelled) return;

            startTransition(() => {
              setRawGrouped((prev) => ({
                ...prev,
                [group_by]: result,
              }));
            });
          },
          { concurrency: 2 },
        );

        if (!cancelled) {
          startTransition(() => setLoading(false));
        }
      } catch (err) {
        if (!cancelled) {
          setError((err as Error).message);
          setLoading(false);
        }
      }
    };

    run();
    return () => {
      cancelled = true;
    };
  }, [preset, overridePreset, groupKeys]);

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
