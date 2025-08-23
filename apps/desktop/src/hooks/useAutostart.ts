import { useCallback, useEffect, useRef, useState } from "react";
import { enable, isEnabled, disable } from "@tauri-apps/plugin-autostart";

export const useAutostart = () => {
  const [enabled, setEnabled] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    (async () => {
      try {
        const on = await isEnabled();
        if (mounted.current) setEnabled(on);
      } catch (e) {
        if (mounted.current)
          setError((e as Error)?.message ?? "Failed to read autostart");
      } finally {
        if (mounted.current) setLoading(false);
      }
    })();

    return () => {
      mounted.current = false;
    };
  }, []);

  const setAutostart = useCallback(
    async (next: boolean): Promise<boolean> => {
      setError(null);
      setLoading(true);
      const prev = enabled;
      setEnabled(next);
      try {
        if (next) {
          await enable();
        } else {
          await disable();
        }
        return true;
      } catch (e) {
        setEnabled(prev);
        setError((e as Error)?.message ?? "Failed to update autostart");
        return false;
      } finally {
        setLoading(false);
      }
    },
    [enabled],
  );

  return { enabled, loading, error, setAutostart };
};
