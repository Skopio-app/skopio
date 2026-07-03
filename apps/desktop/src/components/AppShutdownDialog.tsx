import * as Dialog from "@radix-ui/react-dialog";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { LoaderCircle } from "lucide-react";
import { useEffect, useState } from "react";

const APP_SHUTDOWN_STARTED_EVENT = "app-shutdown-started";

export default function AppShutdownDialog() {
  const [isClosing, setIsClosing] = useState(false);

  useEffect(() => {
    let isMounted = true;
    let unlisten: UnlistenFn | null = null;

    listen(APP_SHUTDOWN_STARTED_EVENT, () => {
      if (isMounted) {
        setIsClosing(true);
      }
    }).then((unsubscribe) => {
      if (isMounted) {
        unlisten = unsubscribe;
      } else {
        unsubscribe();
      }
    });

    return () => {
      isMounted = false;
      unlisten?.();
    };
  }, []);

  return (
    <Dialog.Root open={isClosing}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-1000 bg-background/80 backdrop-blur-sm" />
        <Dialog.Content
          className="fixed left-1/2 top-1/2 z-1001 grid w-[min(90vw,360px)] -translate-x-1/2 -translate-y-1/2 gap-4 rounded-md border border-border bg-background p-5 text-foreground shadow-lg focus:outline-none"
          onEscapeKeyDown={(event) => event.preventDefault()}
          onInteractOutside={(event) => event.preventDefault()}
        >
          <div className="flex items-start gap-3">
            <LoaderCircle
              aria-hidden="true"
              className="mt-0.5 size-5 shrink-0 animate-spin text-muted-foreground"
            />
            <div className="min-w-0 space-y-2">
              <Dialog.Title className="text-base font-semibold leading-6">
                Closing Skopio
              </Dialog.Title>
              <Dialog.Description className="text-sm leading-6 text-muted-foreground">
                Saving recent activity before the app closes. This may take a
                moment.
              </Dialog.Description>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
