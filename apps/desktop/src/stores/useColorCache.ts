import { create } from "zustand";
import { persist, createJSONStorage, StateStorage } from "zustand/middleware";
import { skopioDB } from "@/db/skopioDB";

interface ColorCacheState {
  colorMap: Record<string, string>;
  setColor: (key: string, color: string) => void;
  getColor: (key: string) => string | undefined;
}

const dexieStorage: StateStorage = {
  async getItem(name) {
    const row = await skopioDB.colors.get(name);
    return row?.value ?? null;
  },
  async setItem(name, value) {
    await skopioDB.colors.put({ id: name, value });
  },
  async removeItem(name) {
    await skopioDB.colors.delete(name);
  },
};

export const useColorCache = create<ColorCacheState>()(
  persist(
    (set, get) => ({
      colorMap: {},
      setColor: (key, color) =>
        set((state) => ({
          colorMap: {
            ...state.colorMap,
            [key]: color,
          },
        })),
      getColor: (key) => get().colorMap[key],
    }),
    {
      name: "skopio-color-cache",
      storage: createJSONStorage(() => dexieStorage),
    },
  ),
);
