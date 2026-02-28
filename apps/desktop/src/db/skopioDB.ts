import Dexie, { Table } from "dexie";
import { Layout, ResponsiveLayouts } from "react-grid-layout";
import { CalendarChartData } from "@/types/chart";

export type DashboardBreakpoint = "lg" | "md" | "sm";
export type DashboardLayouts = ResponsiveLayouts<DashboardBreakpoint>;

export interface CachedActivity {
  year: number;
  values: CalendarChartData[];
  updated_at: number;
}

export interface StoredLayout {
  id: string;
  layouts: DashboardLayouts;
  layout?: Layout;
}

export interface ColorRow {
  id: string;
  map: Record<string, string>;
}

class SkopioDB extends Dexie {
  activity!: Table<CachedActivity>;
  layouts!: Table<StoredLayout>;
  colors!: Table<ColorRow>;

  constructor() {
    super("SkopioDB");

    this.version(1).stores({
      activity: "year",
      layouts: "id",
      colors: "id",
    });

    this.version(2)
      .stores({
        activity: "year",
        layouts: "id",
        colors: "id",
      })
      .upgrade((tx) =>
        tx
          .table("layouts")
          .toCollection()
          .modify((entry: StoredLayout) => {
            if (entry.layouts) {
              return;
            }

            entry.layouts = entry.layout ? { lg: entry.layout } : {};
            delete entry.layout;
          }),
      );
  }
}

export const skopioDB = new SkopioDB();
