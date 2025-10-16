import { useEffect, useMemo } from "react";
import {
  BucketSummaryInput,
  BucketTimeSummary,
  commands,
} from "@/types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";
import { useQuery } from "@tanstack/react-query";

// Sum "groupedValues" across all buckets
const sumBucketed = (data: BucketTimeSummary[]): number => {
  const value = data.reduce((sum, item) => {
    const values = Object.values(item.groupedValues ?? {});
    const add = values.reduce((acc, v) => (acc ?? 0) + (v ?? 0), 0);
    return sum + (add ?? 0);
  }, 0);

  return value;
};

// Collect distinct keys across all buckets
const collectKeys = (data: BucketTimeSummary[]) => {
  const set = new Set<string>();
  for (const item of data) {
    console.log("The item: ", item);
    for (const key of Object.keys(item.groupedValues ?? {})) {
      if (key) set.add(key);
      console.log("The key: ", key);
    }
  }
  return [...set];
};

export const useTotalBucketedTime = (): {
  total: number;
  loading: boolean;
  hasBranchData: boolean;
} => {
  const { preset, project, selectedBranches } = usePresetFilter();

  const baseInput = useMemo<BucketSummaryInput>(
    () => ({ preset, projects: [project] }),
    [preset, project],
  );

  const branchInput = useMemo<BucketSummaryInput>(
    () => ({ ...baseInput, groupBy: "branch", branches: selectedBranches }),
    [baseInput, selectedBranches],
  );

  const projectInput = useMemo<BucketSummaryInput>(
    () => ({ ...baseInput, groupBy: "project" }),
    [baseInput],
  );

  const { data, isPending, isFetching } = useQuery({
    queryKey: [
      "bucketedSummary",
      {
        preset,
        project,
        selectedBranches,
      },
    ],
    queryFn: async () => {
      const branchData = await commands.fetchBucketedSummary(branchInput);

      if (branchData.length > 0) {
        return { usedBranch: true as const, buckets: branchData };
      }

      const projectData = await commands.fetchBucketedSummary(projectInput);
      return { usedBranch: false as const, buckets: projectData };
    },
    staleTime: 60_000,
    retry: 2,
    enabled: Boolean(project),
  });

  useEffect(() => {
    if (!data?.usedBranch) return;
    const keys = collectKeys(data.buckets);
    console.log("The keys: ", keys);
    usePresetFilter.setState({ branches: keys });
  }, [data]);

  const loading = isPending || isFetching;
  const total = sumBucketed(data?.buckets ?? []);
  const hasBranchData = Boolean(data?.usedBranch);
  return { total, loading, hasBranchData };
};
