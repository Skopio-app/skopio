import { useEffect, useState } from "react";
import { Layout, ResponsiveLayouts } from "react-grid-layout";
import {
  DashboardBreakpoint,
  DashboardLayouts,
  skopioDB,
  StoredLayout,
} from "@/db/skopioDB";

const normalizeLayouts = (
  entry: StoredLayout | undefined,
  fallback: DashboardLayouts,
): DashboardLayouts => {
  if (!entry) {
    return fallback;
  }

  if (entry.layouts) {
    return entry.layouts;
  }

  if (entry.layout) {
    return { ...fallback, lg: entry.layout };
  }

  return fallback;
};

export const usePersistentLayout = (
  key: string,
  defaultLayouts: DashboardLayouts,
) => {
  const [layouts, setLayouts] = useState<DashboardLayouts>(defaultLayouts);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    skopioDB.layouts.get(key).then((entry) => {
      setLayouts(normalizeLayouts(entry, defaultLayouts));
      setLoaded(true);
    });
  }, [defaultLayouts, key]);

  const saveLayouts = (newLayouts: ResponsiveLayouts<DashboardBreakpoint>) => {
    const nextLayouts = newLayouts as DashboardLayouts;

    setLayouts(nextLayouts);
    skopioDB.layouts.put({ id: key, layouts: nextLayouts });
  };

  const currentLayout: Layout = layouts.lg ?? defaultLayouts.lg ?? [];

  return { layout: currentLayout, layouts, saveLayouts, loaded };
};
