import { useEffect, useState } from "react";
import { TrackedApp, commands } from "../types/tauri.gen";

export const useOpenApps = () => {
  const [apps, setApps] = useState<TrackedApp[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<unknown>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setLoading(true);
        const res = await commands.getOpenApps();
        if (!cancelled) setApps(res ?? []);
      } catch (e) {
        setError(e);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  return {
    apps,
    loading,
    error,
  };
};
