import { create } from "zustand";
import { TimeRangePreset } from "../../../types/tauri.gen";

interface PresetFilterState {
  preset: TimeRangePreset;
  setPreset: (preset: TimeRangePreset) => void;
}

export const usePresetFilter = create<PresetFilterState>((set) => ({
  preset: "today",
  setPreset: (preset) => set({ preset }),
}));
