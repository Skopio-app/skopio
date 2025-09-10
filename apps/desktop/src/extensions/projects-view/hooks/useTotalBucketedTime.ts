import { startTransition, useEffect, useState } from "react";
import { BucketSummaryInput, commands } from "@/types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";

export const useTotalBucketedTime = (): {
  total: number;
  loading: boolean;
  hasBranchData: boolean;
} => {
  const [total, setTotal] = useState<number>(0);
  const [loading, setLoading] = useState<boolean>(true);
  const [hasBranchData, setHasBranchData] = useState(true);
  const { preset, project, selectedBranches } = usePresetFilter();

  useEffect(() => {
    let cancelled = false;
    const fetchData = async () => {
      const baseInput: BucketSummaryInput = {
        preset,
        projects: [project],
      };

      const inputWithBranch: BucketSummaryInput = {
        ...baseInput,
        groupBy: "branch",
        branches: selectedBranches,
      };

      try {
        const branchData = await commands.fetchBucketedSummary(inputWithBranch);
        if (cancelled) return;

        if (branchData.length > 0) {
          const allKeys = new Set<string>();
          for (const item of branchData) {
            Object.keys(item.grouped_values ?? {}).forEach((key) => {
              if (key) allKeys.add(key);
            });
          }

          usePresetFilter.setState({ branches: [...allKeys] });
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
  }, [preset, project, selectedBranches]);

  return { total, loading, hasBranchData };
};
