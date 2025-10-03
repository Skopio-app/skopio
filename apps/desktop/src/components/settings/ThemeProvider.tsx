import { Theme, commands, events } from "@/types/tauri.gen";
import { ThemeContext } from "@/utils/theme";
import { useEffect, useRef, useState } from "react";
import { setTheme as setTauriTheme } from "@tauri-apps/api/app";
import { UnlistenFn } from "@tauri-apps/api/event";

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

  const applyThemeToDOM = (t: Theme, isDark: boolean) => {
    const root = document.documentElement;
    root.classList.remove("light", "dark");
    root.classList.add(t === "system" ? (isDark ? "dark" : "light") : t);
  };

  const applyTheme = async (t: Theme) => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    await setTauriTheme(t === "system" ? null : t);
    applyThemeToDOM(t, mql.matches);
  };

  useEffect(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    let unlisten: UnlistenFn;

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
      if (theme === "system") {
        applyThemeToDOM("system", e.matches);
      }
    };
    mql.addEventListener("change", onChange);

    (async () => {
      unlisten = await events.theme.listen((e) => {
        setTheme(e.payload);
      });
    })();

    return () => {
      mql.removeEventListener("change", onChange);
      if (unlisten) unlisten();
    };
  }, []);

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
  }, [theme, hydrated]);

  const value = {
    theme,
    setTheme: (t: Theme) => setTheme(t),
  };

  return (
    <ThemeContext.Provider {...props} value={value}>
      {children}
    </ThemeContext.Provider>
  );
}
