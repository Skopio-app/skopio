import {
  commands,
  Group,
  InsightQueryPayload,
  InsightResult,
} from "@/types/tauri.gen";
import { useYearFilter } from "../stores/useYearFilter";
import { useQuery } from "@tanstack/react-query";
import { toast } from "sonner";

export const useTopNInsights = ({
  groupBy,
  limit = 3,
}: {
  groupBy: Group;
  limit?: number;
}) => {
  const { year } = useYearFilter();

  type TopN = [string, number][];
  const { data, isLoading, error } = useQuery({
    queryKey: ["topN", year, limit, groupBy],
    queryFn: async (): Promise<InsightResult> => {
      const query: InsightQueryPayload = {
        insightType: "topN",
        insightRange: year,
        limit,
        groupBy,
      };

      return commands.fetchInsights(query);
    },
    select: (res): TopN => ("topN" in res ? res.topN : []),
    enabled: Boolean(year),
  });

  if (error) {
    toast.error(`Failed to fetch topN insights: ${(error as Error).message}`);
  }
  const parsed = data ?? [];

  return { data: parsed, loading: isLoading };
};
