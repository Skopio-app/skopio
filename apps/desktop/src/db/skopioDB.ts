import Dexie, { Table } from "dexie";
import { Layout } from "react-grid-layout";
import { CalendarChartData } from "@/types/chart";

export interface CachedActivity {
  year: number;
  values: CalendarChartData[];
  updated_at: number;
}

export interface StoredLayout {
  id: string;
  layout: Layout[];
}

export interface ColorRow {
  id: string;
  value: string;
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
  }
}

export const skopioDB = new SkopioDB();
