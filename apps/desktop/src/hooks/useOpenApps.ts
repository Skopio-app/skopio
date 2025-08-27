import { useEffect, useState } from "react";
import { OpenApp, commands } from "../types/tauri.gen";

export const useOpenApps = () => {
  const [apps, setApps] = useState<OpenApp[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<unknown>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setLoading(true);
        const apps = await commands.getOpenApps();
        if (!cancelled) setApps(apps ?? []);
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
