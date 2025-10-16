import { commands, SummaryQueryInput } from "../../../types/tauri.gen";
import { useQuery } from "@tanstack/react-query";

export const useTotalTime = (
  start: Date,
  end: Date,
): { total: number; loading: boolean } => {
  const {
    data: total = 0,
    isPending,
    isFetching,
  } = useQuery({
    queryKey: ["totalTime", start.toISOString(), end.toISOString()],
    queryFn: async (): Promise<number> => {
      const input: SummaryQueryInput = {
        start: start.toISOString(),
        end: end.toISOString(),
      };
      return commands.fetchTotalTime(input);
    },
    enabled: !!start && !!end,
  });

  const loading = isPending || isFetching;

  return { total, loading };
};
