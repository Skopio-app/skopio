import { commands } from "@/types/tauri.gen";
import type { Theme } from "@/types/tauri.gen";
import { ThemeContext } from "@/utils/theme";
import type { ResolvedTheme } from "@/utils/theme";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { setTheme as setTauriTheme } from "@tauri-apps/api/app";
import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";

const THEME_CHANGED_EVENT = "theme-changed";
const THEMES = ["light", "dark", "system"] satisfies Theme[];

const isTheme = (value: unknown): value is Theme =>
  THEMES.includes(value as Theme);

type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: Theme;
};

const getSystemTheme = (): ResolvedTheme =>
  window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";

const resolveTheme = (theme: Theme, systemTheme: ResolvedTheme) =>
  theme === "system" ? systemTheme : theme;

export default function ThemeProvider({
  children,
  defaultTheme = "system",
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(defaultTheme);
  const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(getSystemTheme);
  const [hydrated, setHydrated] = useState(false);
  const isInitialThemeChange = useRef(true);
  const externalThemeChange = useRef(false);
  const themeRef = useRef(theme);
  const resolvedTheme = resolveTheme(theme, systemTheme);

  const applyThemeToDOM = useCallback((resolvedTheme: ResolvedTheme) => {
    const root = document.documentElement;
    root.classList.remove("light", "dark");
    root.classList.add(resolvedTheme);
  }, []);

  const applyTheme = useCallback(
    async (theme: Theme, resolvedTheme: ResolvedTheme) => {
      await setTauriTheme(theme === "system" ? null : theme);
      applyThemeToDOM(resolvedTheme);
    },
    [applyThemeToDOM],
  );

  useEffect(() => {
    themeRef.current = theme;
  }, [theme]);

  useEffect(() => {
    (async () => {
      try {
        const cfg = await commands.getConfig();
        const initial = cfg.theme;
        const initialSystemTheme = getSystemTheme();
        setSystemTheme(initialSystemTheme);
        setTheme(initial);
        await applyTheme(initial, resolveTheme(initial, initialSystemTheme));
      } finally {
        setHydrated(true);
      }
    })();
  }, [applyTheme]);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<Theme>(THEME_CHANGED_EVENT, (event) => {
        const nextTheme = event.payload;
        if (!isTheme(nextTheme) || nextTheme === themeRef.current) return;

        externalThemeChange.current = true;
        setTheme(nextTheme);
      });
    })();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = (e: MediaQueryListEvent) =>
      setSystemTheme(e.matches ? "dark" : "light");

    mql.addEventListener("change", onChange);

    return () => {
      mql.removeEventListener("change", onChange);
    };
  }, []);

  useEffect(() => {
    if (!hydrated) {
      return;
    }

    void applyTheme(theme, resolvedTheme);
  }, [theme, resolvedTheme, hydrated, applyTheme]);

  useEffect(() => {
    if (!hydrated) {
      return;
    }

    if (isInitialThemeChange.current) {
      isInitialThemeChange.current = false;
      return;
    }

    (async () => {
      if (externalThemeChange.current) {
        externalThemeChange.current = false;
        return;
      }

      await commands.setTheme(theme);
      await emit<Theme>(THEME_CHANGED_EVENT, theme);
    })();
  }, [theme, hydrated]);

  const value = useMemo(
    () => ({
      theme,
      resolvedTheme,
      setTheme,
    }),
    [theme, resolvedTheme],
  );

  return (
    <ThemeContext.Provider {...props} value={value}>
      {children}
    </ThemeContext.Provider>
  );
}
