import { useCallback, useEffect, useRef, useState } from "react";
import { loadShortcut, updateGlobalShortcut } from "../utils/shortcut";
import { acceleratorToUI, uiComboToAccelerator } from "../utils/hotkey";

export const useGlobalShortcut = () => {
  const [shortcut, setShortcut] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const debounceRef = useRef<number | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    (async () => {
      try {
        const accel = await loadShortcut();
        const ui = acceleratorToUI(accel);
        if (mounted.current) setShortcut(ui);
      } catch (e) {
        if (mounted.current)
          setError((e as Error)?.message ?? "Failed to load shortcut");
      } finally {
        if (mounted.current) setLoading(false);
      }
    })();
    return () => {
      mounted.current = false;
    };
  }, []);

  const setAndPersist = useCallback((nextUI: string) => {
    setError(null);
    setShortcut(nextUI);
    if (debounceRef.current) window.clearTimeout(debounceRef.current);
    debounceRef.current = window.setTimeout(async () => {
      setBusy(true);
      try {
        const accel = uiComboToAccelerator(nextUI);
        await updateGlobalShortcut(accel);
      } catch (e) {
        setError((e as Error)?.message ?? "Failed to update global shortcut");
      } finally {
        setBusy(false);
      }
    }, 250) as number;
  }, []);

  return { shortcut, setAndPersist, loading, busy, error };
};
