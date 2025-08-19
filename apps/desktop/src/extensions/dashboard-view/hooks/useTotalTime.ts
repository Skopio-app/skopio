import { startTransition, useEffect, useState } from "react";
import { commands, SummaryQueryInput } from "../../../types/tauri.gen";

export const useTotalTime = (
  start: Date,
  end: Date,
): { total: number; loading: boolean } => {
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;

    startTransition(() => {
      const input: SummaryQueryInput = {
        start: start.toISOString(),
        end: end.toISOString(),
      };

      setLoading(true);

      commands
        .fetchTotalTime(input)
        .then((result) => {
          if (!cancelled) {
            setTotal(result);
            setLoading(false);
          }
        })
        .catch(() => {
          if (!cancelled) setLoading(false);
        });
    });

    return () => {
      cancelled = true;
    };
  }, [start, end]);

  return { total, loading };
};
