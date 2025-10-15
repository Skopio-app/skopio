import { Theme, commands } from "@/types/tauri.gen";
import { ThemeContext } from "@/utils/theme";
import { useCallback, useEffect, useRef, useState } from "react";
import { setTheme as setTauriTheme } from "@tauri-apps/api/app";

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
  const themeRef = useRef(theme);
  themeRef.current = theme;

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
    const mql = window.matchMedia("(prefers-color-scheme: dark)");

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

    const onChange = (e: MediaQueryListEvent) => {
      if (themeRef.current === "system") {
        applyThemeToDOM("system", e.matches);
      }
    };
    mql.addEventListener("change", onChange);

    return () => {
      mql.removeEventListener("change", onChange);
    };
  }, [applyTheme, applyThemeToDOM]);

  useEffect(() => {
    if (isInitialMount.current) {
      isInitialMount.current = false;
      return;
    }

    if (hydrated) {
      (async () => {
        await applyTheme(theme);
        await commands.setTheme(theme);
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
