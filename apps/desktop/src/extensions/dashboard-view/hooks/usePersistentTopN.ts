import { useEffect, useState } from "react";

export const usePersistentTopN = (key: string, defaultValue: number) => {
  const [topN, setTopN] = useState<number>(() => {
    try {
      const saved = localStorage.getItem(key);
      if (saved === null) return defaultValue;
      if (!Number.isInteger(Number(saved))) {
        return defaultValue;
      }
      return parseInt(saved, 10);
    } catch {
      return defaultValue;
    }
  });

  useEffect(() => {
    localStorage.setItem(key, String(topN));
  }, [key, topN]);

  return [topN, setTopN] as const;
};
