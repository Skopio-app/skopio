import { create } from "zustand";
import { TimeRangePreset } from "../../../types/tauri.gen";

interface PresetFilterState {
  preset: TimeRangePreset;
  project: string;
  setPreset: (preset: TimeRangePreset) => void;
  setProject: (project: string) => void;
}

export const usePresetFilter = create<PresetFilterState>((set) => ({
  preset: "today",
  project: "",
  setPreset: (preset) => set({ preset }),
  setProject: (project) => set({ project }),
}));
