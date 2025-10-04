import { Theme } from "@/types/tauri.gen";
import { createContext, useContext } from "react";

export type ThemeProviderState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
};

export const ThemeContext = createContext<ThemeProviderState | undefined>(
  undefined,
);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be within a ThemeProvider");
  return context;
};
