import { create } from "zustand";
import { persist, createJSONStorage, StateStorage } from "zustand/middleware";
import { skopioDB } from "@/db/skopioDB";

interface ColorCacheState {
  colorMap: Record<string, string>;
  setColor: (key: string, color: string) => void;
  getColor: (key: string) => string | undefined;
}

const colorStorage: StateStorage = {
  async getItem(name) {
    const row = await skopioDB.colors.get(name);
    if (!row) return null;
    const payload = {
      state: { colorMap: row.map ?? {} },
      version: 0,
    };
    return JSON.stringify(payload);
  },
  async setItem(name, value) {
    try {
      const parsed = JSON.parse(value ?? "{}");
      const colorMap: Record<string, string> = parsed?.state?.colorMap ?? {};

      await skopioDB.colors.put({ id: name, map: colorMap });
    } catch (e) {
      console.error("Error getting color item: ", e);
    }
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
      storage: createJSONStorage(() => colorStorage),
    },
  ),
);
