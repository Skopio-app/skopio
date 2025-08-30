import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { commands } from "../types/tauri.gen";
import {
  register as registerShortcut,
  unregister as unregisterShortcut,
} from "@tauri-apps/plugin-global-shortcut";
import { emit } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const SHORTCUT_EVENT = "global-shortcut";

export const goBack = () => window.history.back();
export const goForward = () => window.history.forward();
export const reloadWindow = () => window.location.reload();

const isEditableTarget = (target: EventTarget | null): boolean => {
  const el = target as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName.toLowerCase();
  if (
    tag === "input" ||
    tag === "textarea" ||
    (el as HTMLInputElement).isContentEditable
  )
    return true;
  return false;
};

export const useHistoryControls = () => {
  const read = () => {
    const idx =
      (window.history.state && (window.history.state as any).idx) ?? 0;
    const len = window.history.length;
    return { idx, len };
  };

  const [{ idx, len }, setState] = useState(read);

  useEffect(() => {
    const push = window.history.pushState;
    const replace = window.history.replaceState;

    if (!(window as any).__skopio_history_patched) {
      (window as any).__skopio_history_patched = true;
      window.history.pushState = function (...args) {
        const ret = push.apply(this, args);
        window.dispatchEvent(new Event("skopio:pushstate"));
        return ret;
      };
      window.history.replaceState = function (...args) {
        const ret = replace.apply(this, args);
        window.dispatchEvent(new Event("skopio:replacestate"));
        return ret;
      };
    }

    const update = () => setState(read());
    window.addEventListener("popstate", update);
    window.addEventListener("skopio:pushstate", update);
    window.addEventListener("skopio:replacestate", update);

    update();

    return () => {
      window.removeEventListener("popstate", update);
      window.removeEventListener("skopio:pushstate", update);
      window.removeEventListener("skopio:replacestate", update);
    };
  }, []);

  return useMemo(
    () => ({
      canGoBack: idx > 0,
      canGoForward: idx < len - 1,
      idx,
      len,
    }),
    [idx, len],
  );
};

export const loadShortcut = async (): Promise<string> => {
  try {
    const shortcut = (await commands.getConfig()).globalShortcut;
    return shortcut;
  } catch (e) {
    console.error("Failed to load shortcut: ", e);
    return "";
  }
};

export const saveShorcut = async (shortcut: string): Promise<void> => {
  try {
    await commands.setGlobalShortcut(shortcut);
  } catch (e) {
    console.log("Failed to save shortcut command: ", e);
  }
};

export const focusApp = async () => {
  try {
    const main = (await WebviewWindow.getByLabel("main")) ?? getCurrentWindow();
    await main.show();
    await main.setFocus();
    await main.unminimize();
  } catch (e) {
    console.error("Focusing app failed: ", e);
  }
};

export const updateGlobalShortcut = async (
  newShortcut: string,
): Promise<void> => {
  const currentShortcut = await loadShortcut();
  if (newShortcut === currentShortcut) {
    return;
  }

  try {
    if (currentShortcut) {
      await unregisterShortcut(currentShortcut);
    }

    if (newShortcut) {
      try {
        await registerShortcut(newShortcut, (event) => {
          if (event.state === "Pressed") {
            emit(SHORTCUT_EVENT);
          }
        });
      } catch (e) {
        console.log("Error registering shortcut: ", e);
      }
    }

    await saveShorcut(newShortcut);
  } catch (e) {
    console.error("Failed to update shortcut: ", e);
  }
};

export const initializeGlobalShortcut = async (): Promise<void> => {
  try {
    const shortcutToRegister = await loadShortcut();
    if (shortcutToRegister) {
      await unregisterShortcut(shortcutToRegister);
    }
    if (shortcutToRegister) {
      await registerShortcut(shortcutToRegister, (event) => {
        if (event.state === "Pressed") {
          emit(SHORTCUT_EVENT);
        }
      });
    }
  } catch (e) {
    // error seems to appear even though the shortcut is initialized successfully
    console.error("Failed to initialize global shortcut: ", e);
  }
};

export const useGlobalShortcutListener = () => {
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;
    const onKeyDown = async (e: KeyboardEvent) => {
      if (isEditableTarget(e.target)) return;

      const key = e.key;
      const meta = e.metaKey;
      const shift = e.shiftKey;
      const ctrl = e.ctrlKey;
      const alt = e.altKey;

      if (meta && key === "[") {
        e.preventDefault();
        goBack();
        return;
      }
      if (meta && key === "]") {
        e.preventDefault();
        goForward();
        return;
      }
      if (meta && key === ",") {
        e.preventDefault();
        await commands.showWindow("settings");
        return;
      }
      if (meta && shift && key === "r") {
        e.preventDefault();
        reloadWindow();
        return;
      }

      // Windows specific
      if (alt && key === "ArrowLeft") {
        e.preventDefault();
        goBack();
        return;
      }
      if (alt && key === "ArrowRight") {
        e.preventDefault();
        goForward();
        return;
      }
      if (ctrl && (key === "r" || key === "R")) {
        e.preventDefault();
        reloadWindow();
        return;
      }
    };

    window.addEventListener("keydown", onKeyDown, { capture: true });

    (async () => {
      try {
        await initializeGlobalShortcut();
        unlisten = await listen(SHORTCUT_EVENT, async () => {
          await focusApp();
        });
      } catch (e) {
        console.error("Failed to set up global shortcut listener: ", e);
      }
    })();

    return () => {
      window.removeEventListener("keydown", onKeyDown, {
        capture: true,
      } as any);
      if (unlisten) unlisten();
    };
  }, []);
};
