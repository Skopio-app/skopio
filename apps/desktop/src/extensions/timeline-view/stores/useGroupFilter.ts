import { create } from "zustand";
import { Group } from "../../../types/tauri.gen";

interface GroupFilterState {
  group: Group;
  setGroup: (group: Group) => void;
}

export const useGroupFilter = create<GroupFilterState>((set) => ({
  group: "category",
  setGroup: (group) => set({ group }),
}));
