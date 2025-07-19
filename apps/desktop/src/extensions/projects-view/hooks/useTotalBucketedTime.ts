import { startTransition, useEffect, useState } from "react";
import {
  BucketedSummaryInput,
  commands,
  TimeRangePreset,
} from "../../../types/tauri.gen";

export const useTotalBucketedTime = (
  preset: TimeRangePreset,
  projectName: string,
  selectedBranches: string[] | null,
): {
  total: number;
  loading: boolean;
  hasBranchData: boolean;
  branches: string[];
} => {
  const [total, setTotal] = useState<number>(0);
  const [loading, setLoading] = useState<boolean>(true);
  const [hasBranchData, setHasBranchData] = useState(false);
  const [branches, setBranches] = useState<string[]>([]);

  useEffect(() => {
    let cancelled = false;
    const fetchData = async () => {
      const baseInput: BucketedSummaryInput = {
        preset,
        project_names: [projectName],
        include_afk: false,
      };

      const inputWithBranch: BucketedSummaryInput = {
        ...baseInput,
        group_by: "branch",
        ...(selectedBranches ? { branch_names: selectedBranches } : {}),
      };

      try {
        setLoading(true);

        const branchData = await commands.fetchBucketedSummary(inputWithBranch);
        if (cancelled) return;

        if (branchData.length > 0) {
          const allKeys = new Set<string>();
          for (const item of branchData) {
            Object.keys(item.grouped_values ?? {}).forEach((key) => {
              if (key) allKeys.add(key);
            });
          }

          setBranches([...allKeys]);
          setHasBranchData(branches.length > 0);

          const branchTotal = branchData.reduce((sum, item) => {
            const values = Object.values(item.grouped_values ?? {});
            return (
              sum +
              values
                .filter((v): v is number => typeof v === "number")
                .reduce((a, b) => a + b, 0)
            );
          }, 0);

          setTotal(branchTotal);
        } else {
          const fallbackData = await commands.fetchBucketedSummary({
            ...baseInput,
            group_by: "project",
          });
          if (cancelled) return;

          const fallbackTotal = fallbackData.reduce((sum, item) => {
            const values = Object.values(item.grouped_values ?? {});
            return (
              sum +
              values
                .filter((v): v is number => typeof v === "number")
                .reduce((a, b) => a + b, 0)
            );
          }, 0);
          setTotal(fallbackTotal);
          setHasBranchData(false);
        }
      } catch (err) {
        console.error("Error fetching summary: ", err);
      } finally {
        if (!cancelled) setLoading(false);
      }
    };

    startTransition(() => {
      void fetchData();
    });
    return () => {
      cancelled = true;
    };
  }, [preset, projectName, selectedBranches?.join(",")]);

  return { total, loading, hasBranchData, branches };
};
