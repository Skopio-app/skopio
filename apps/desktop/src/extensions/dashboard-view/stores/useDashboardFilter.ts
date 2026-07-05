import { create } from "zustand";
import { TimeRangePreset } from "@/types/tauri.gen";
import { DateRangeType, getRangeDates } from "@/utils/time";

interface DashboardFilterState {
  preset: TimeRangePreset;
  startDate: Date;
  endDate: Date;
  setPreset: (preset: TimeRangePreset) => void;
  setDateRange: (startDate: Date, endDate: Date) => void;
}

const [defaultStartDate, defaultEndDate] = getRangeDates(DateRangeType.Today);

export const useDashboardFilter = create<DashboardFilterState>((set) => ({
  preset: "today",
  startDate: defaultStartDate,
  endDate: defaultEndDate,
  setPreset: (preset) => set({ preset }),
  setDateRange: (startDate, endDate) => set({ startDate, endDate }),
}));
