import { Theme, commands } from "@/types/tauri.gen";
import { ThemeContext } from "@/utils/theme";
import { useCallback, useEffect, useRef, useState } from "react";
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

export default function ThemeProvider({
  children,
  defaultTheme = "system",
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(defaultTheme);
  const [hydrated, setHydrated] = useState(false);
  const isInitialMount = useRef(true);
  const externalThemeChange = useRef(false);
  const themeRef = useRef(theme);

  const applyThemeToDOM = useCallback((theme: Theme, isDark: boolean) => {
    const root = document.documentElement;
    root.classList.remove("light", "dark");
    root.classList.add(
      theme === "system" ? (isDark ? "dark" : "light") : theme,
    );
  }, []);

  const applyTheme = useCallback(
    async (theme: Theme) => {
      const mql = window.matchMedia("(prefers-color-scheme: dark)");
      await setTauriTheme(theme === "system" ? null : theme);
      applyThemeToDOM(theme, mql.matches);
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
        setTheme(initial);
        await applyTheme(initial);
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
    const onChange = (e: MediaQueryListEvent) => {
      if (theme === "system") {
        applyThemeToDOM("system", e.matches);
      }
    };

    mql.addEventListener("change", onChange);

    return () => {
      mql.removeEventListener("change", onChange);
    };
  }, [theme, applyThemeToDOM]);

  useEffect(() => {
    if (isInitialMount.current) {
      isInitialMount.current = false;
      return;
    }

    if (hydrated) {
      (async () => {
        await applyTheme(theme);
        if (externalThemeChange.current) {
          externalThemeChange.current = false;
          return;
        }

        await commands.setTheme(theme);
        await emit<Theme>(THEME_CHANGED_EVENT, theme);
      })();
    }
  }, [theme, hydrated, applyTheme]);

  const value = {
    theme,
    setTheme: (theme: Theme) => setTheme(theme),
  };

  return (
    <ThemeContext.Provider {...props} value={value}>
      {children}
    </ThemeContext.Provider>
  );
}
