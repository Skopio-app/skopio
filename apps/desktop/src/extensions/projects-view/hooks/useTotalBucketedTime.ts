import { startTransition, useEffect, useState } from "react";
import { BucketedSummaryInput, commands } from "../../../types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";

export const useTotalBucketedTime = (
  selectedBranches: string[] | null,
): {
  total: number;
  loading: boolean;
  hasBranchData: boolean;
  branches: string[];
} => {
  const [total, setTotal] = useState<number>(0);
  const [loading, setLoading] = useState<boolean>(true);
  const [hasBranchData, setHasBranchData] = useState(true);
  const [branches, setBranches] = useState<string[]>([]);
  const { preset, project } = usePresetFilter();

  useEffect(() => {
    let cancelled = false;
    const fetchData = async () => {
      const baseInput: BucketedSummaryInput = {
        preset,
        projects: [project],
      };

      const inputWithBranch: BucketedSummaryInput = {
        ...baseInput,
        groupBy: "branch",
        ...(selectedBranches ? { branch_names: selectedBranches } : null),
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
          setHasBranchData(allKeys.size > 0);

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
            groupBy: "project",
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
  }, [preset, project, selectedBranches?.join(",")]);

  return { total, loading, hasBranchData, branches };
};
