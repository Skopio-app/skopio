import { create } from "zustand";
import { TimeRangePreset } from "@/types/tauri.gen";

interface PresetFilterState {
  preset: TimeRangePreset;
  project: string;
  branches: string[];
  selectedBranches: string[] | null;
  setPreset: (preset: TimeRangePreset) => void;
  setProject: (project: string) => void;
  setBranches: (branches: string[]) => void;
  setSelectedBranch: (branches: string[] | null) => void;
}

export const usePresetFilter = create<PresetFilterState>((set) => ({
  preset: "today",
  project: "",
  branches: [""],
  selectedBranches: null,
  setPreset: (preset) => set({ preset }),
  setProject: (project) => set({ project }),
  setBranches: (branches) => set({ branches }),
  setSelectedBranch: (selectedBranches) => set({ selectedBranches }),
}));
