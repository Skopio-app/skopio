import { commands, events, ServerStatus } from "@/types/tauri.gen";
import { UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export const useServerStatus = () => {
  const [status, setStatus] = useState<ServerStatus>({ state: "offline" });

  useEffect(() => {
    let unlisten: UnlistenFn;

    (async () => {
      const current = await commands.getServerStatus();
      setStatus(current);

      unlisten = await events.serverStatus.listen((e) => {
        setStatus(e.payload);
      });
    })();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return status;
};
