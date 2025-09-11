import { commands, events, ServerStatus } from "@/types/tauri.gen";
import { UnlistenFn } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";

export const useServerStatus = () => {
  const [status, setStatus] = useState<ServerStatus>({ state: "offline" });

  useEffect(() => {
    let unlisten: UnlistenFn;

    (async () => {
      const current = await commands.getServerStatus();
      console.log("Current: ", current);
      setStatus(current);

      const appWindow = new WebviewWindow("main");
      unlisten = await events.serverStatus(appWindow).listen((e) => {
        console.log("The payload: ", e.payload);
        setStatus(e.payload);
      });
    })();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return status;
};
