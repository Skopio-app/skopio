import { useCallback, useEffect, useState } from "react";
import { OpenApp, commands } from "@/types/tauri.gen";

export const useOpenApps = () => {
  const [apps, setApps] = useState<OpenApp[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<unknown>(null);

  const fetch = useCallback(async () => {
    try {
      setLoading(true);
      const apps = await commands.getOpenApps();
      setApps(apps ?? []);
    } catch (e) {
      setError(e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void fetch();
  }, [fetch]);

  return {
    apps,
    loading,
    error,
    fetch,
  };
};
