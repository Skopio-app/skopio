import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@skopio/ui";
import { Outlet } from "react-router";
import { Toaster } from "sonner";
import {
  goBack,
  goForward,
  reloadWindow,
  useGlobalShortcutListener,
  useHistoryControls,
} from "@/utils/shortcut";
import { isDev } from "@/utils/environment";
import { commands } from "@/types/tauri.gen";
import PermissionsDialog from "@/components/settings/PermissionsDialog";

function App() {
  useGlobalShortcutListener();
  const { canGoBack, canGoForward } = useHistoryControls();

  return (
    <>
      <Toaster richColors />
      <PermissionsDialog />
      <ContextMenu>
        <ContextMenuTrigger className="block min-h-screen w-full">
          <Outlet />
        </ContextMenuTrigger>

        <ContextMenuContent className="w-42 h-32">
          <ContextMenuItem
            className="text-xs"
            disabled={!canGoBack}
            onClick={goBack}
          >
            Back
            <ContextMenuShortcut>⌘[</ContextMenuShortcut>
          </ContextMenuItem>
          <ContextMenuItem
            className="text-xs"
            disabled={!canGoForward}
            onClick={goForward}
          >
            Forward
            <ContextMenuShortcut>⌘]</ContextMenuShortcut>
          </ContextMenuItem>
          <ContextMenuItem className="text-xs" onClick={reloadWindow}>
            Reload
            <ContextMenuShortcut>⌘⇧R</ContextMenuShortcut>
          </ContextMenuItem>
          {isDev && (
            <ContextMenuItem
              inset
              className="text-xs"
              disabled={!isDev()}
              onClick={() => commands.openDevtools()}
            >
              Inspect Element
            </ContextMenuItem>
          )}
        </ContextMenuContent>
      </ContextMenu>
    </>
  );
}

export default App;
