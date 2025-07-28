import { useEffect, useState } from "react";
import { commands, Group, InsightQueryPayload } from "../../../types/tauri.gen";
import { useYearFilter } from "../stores/useYearFilter";
import { toast } from "sonner";

export const useTopNInsights = ({
  groupBy,
  limit = 3,
}: {
  groupBy: Group;
  limit?: number;
}) => {
  const [data, setData] = useState<[string, number][]>([]);
  const [loading, setLoading] = useState(true);
  const { year } = useYearFilter();

  useEffect(() => {
    const fetch = async () => {
      const query: InsightQueryPayload = {
        insightType: "topN",
        insightRange: year,
        limit,
        groupBy,
      };

      try {
        if (year.length === 0) return;
        const result = await commands.fetchInsights(query);
        if ("topN" in result) {
          setData(result.topN);
        }
      } catch (err) {
        toast.error(`Failed to fetch topN insights: ${err}`);
      } finally {
        setLoading(false);
      }
    };

    fetch();
  }, [limit, groupBy, year]);

  return { data, loading };
};
