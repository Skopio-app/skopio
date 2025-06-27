import { Layout } from "react-grid-layout";
import Dexie, { Table } from "dexie";

export interface StoredLayout {
  id: string;
  layout: Layout[];
}

class DashboardLayoutDB extends Dexie {
  layouts!: Table<StoredLayout>;

  constructor() {
    super("DashboardLayoutDB");
    this.version(1).stores({
      layouts: "id",
    });
  }
}

export const dashboardLayoutDB = new DashboardLayoutDB();
