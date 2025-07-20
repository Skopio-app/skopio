import Dexie, { Table } from "dexie";
import { Layout } from "react-grid-layout";
import { CalendarChartData } from "../../../types/types";

export interface CachedActivity {
  year: number;
  values: CalendarChartData[];
  updated_at: number;
}

export interface StoredLayout {
  id: string;
  layout: Layout[];
}

class SkopioDB extends Dexie {
  activity!: Table<CachedActivity>;
  layouts!: Table<StoredLayout>;

  constructor() {
    super("SkopioDB");

    this.version(1).stores({
      activity: "year",
      layouts: "id",
    });
  }
}

export const skopioDB = new SkopioDB();
