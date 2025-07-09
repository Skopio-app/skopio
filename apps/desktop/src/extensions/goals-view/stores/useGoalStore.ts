import { create } from "zustand";
import { commands, Goal, GoalUpdateInput } from "../../../types/tauri.gen";
import { toast } from "sonner";

interface GoalStore {
  goals: Goal[];
  loading: boolean;
  fetchGoals: () => Promise<void>;
  updateGoal: (id: number, updates: GoalUpdateInput) => Promise<void>;
  deleteGoal: (id: number) => Promise<void>;
}

export const useGoalStore = create<GoalStore>((set) => ({
  goals: [],
  loading: false,

  fetchGoals: async () => {
    set({ loading: true });

    try {
      const data = await commands.getGoals();
      set({ goals: data });
    } catch (err) {
      console.error("Failed to fetch goals: ", err);
    } finally {
      set({ loading: false });
    }
  },

  updateGoal: async (id, updates) => {
    try {
      await commands.updateGoal(id, updates);
      await useGoalStore.getState().fetchGoals();
    } catch (err) {
      toast.error(`Error updating goal: ${err}`);
    } finally {
      toast.success("Goal updated successfully!");
    }
  },

  deleteGoal: async (id) => {
    try {
      await commands.removeGoal(id);
      await useGoalStore.getState().fetchGoals();
    } catch (err) {
      toast.error(`Failed to delete goal: ${err}`);
    } finally {
      toast.success("Goal deleted successfully");
    }
  },
}));
