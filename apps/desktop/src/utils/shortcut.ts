import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { commands } from "../types/tauri.gen";
import {
  register as registerShortcut,
  unregister as unregisterShortcut,
} from "@tauri-apps/plugin-global-shortcut";
import { emit } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const SHORTCUT_EVENT = "global-shortcut";

export const loadShortcut = async (): Promise<string> => {
  try {
    const shortcut = (await commands.getConfig()).global_shortcut;
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
      if (unlisten) unlisten();
    };
  }, []);
};
