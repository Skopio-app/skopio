import { create } from "zustand";
import { persist } from "zustand/middleware";

interface ColorCacheState {
  colorMap: Record<string, string>;
  setColor: (key: string, color: string) => void;
  getColor: (key: string) => string | undefined;
}

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
      name: "skopio-color-cache", // storage key in localStorage
    },
  ),
);
