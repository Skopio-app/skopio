import { create } from "zustand";

interface YearFilterState {
  year: string;
  setYear: (year: string) => void;
}

export const useYearFilter = create<YearFilterState>((set) => ({
  year: "",
  setYear: (year) => set({ year }),
}));
