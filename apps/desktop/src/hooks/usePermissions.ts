import { useCallback, useEffect, useState } from "react";
import {
  commands,
  PermissionKind,
  PermissionSummary,
} from "../types/tauri.gen";

export const usePermissions = () => {
  const [summary, setSummary] = useState<PermissionSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<null | PermissionKind>(null);
  const [error, setError] = useState<string | null>(null);

  const check = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const summary = await commands.getPermissions();
      setSummary(summary);
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void check();
  }, [check]);

  const requestAccessibility = async () => {
    setBusy("accessibility");
    setError(null);
    try {
      const result = await commands.requestAccessibilityPermission();
      return result;
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setBusy(null);
    }
  };

  const requestInputMonitoring = async () => {
    setBusy("inputMonitoring");
    setError(null);
    try {
      const result = await commands.requestInputMonitoringPermission();
      return result;
    } catch (e: any) {
      setError(e?.message ?? String(e));
    } finally {
      setBusy(null);
    }
  };

  const openSettings = async (kind: "accessibility" | "inputMonitoring") => {
    try {
      await commands.openPermissionSettings(kind);
    } catch (e) {
      console.warn("Error opening settings: ", e);
    }
  };

  return {
    summary,
    loading,
    busy,
    error,
    check,
    requestAccessibility,
    requestInputMonitoring,
    openSettings,
  };
};
