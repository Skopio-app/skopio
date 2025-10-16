import { create } from "zustand";

interface YearFilterState {
  year: string;
  setYear: (year: string) => void;
}

export const useYearFilter = create<YearFilterState>((set) => ({
  year: new Date().getFullYear().toString(),
  setYear: (year) => set({ year }),
}));
