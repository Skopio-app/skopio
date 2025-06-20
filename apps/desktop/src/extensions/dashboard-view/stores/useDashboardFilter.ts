import { create } from "zustand";

interface DashboardFilterState {
  startDate: Date;
  endDate: Date;
  setStartDate: (date: Date) => void;
  setEndDate: (date: Date) => void;
}

export const useDashboardFilter = create<DashboardFilterState>((set) => ({
  startDate: new Date(),
  endDate: new Date(),
  setStartDate: (startDate) => set({ startDate }),
  setEndDate: (endDate) => set({ endDate }),
}));
