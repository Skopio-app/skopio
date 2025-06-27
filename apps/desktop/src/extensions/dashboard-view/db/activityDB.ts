import Dexie, { Table } from "dexie";

export interface CachedActivity {
  day: string;
  value: number;
  year: number;
  updated_at: number;
}

class ActivityDB extends Dexie {
  activity!: Table<CachedActivity>;

  constructor() {
    super("ActivityDB");
    this.version(1).stores({
      activity: "&day, year, value, updated_at",
    });
  }
}

export const activityDB = new ActivityDB();
