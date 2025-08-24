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
} from "./utils/shortcut";

function App() {
  useGlobalShortcutListener();
  const { canGoBack, canGoForward } = useHistoryControls();

  return (
    <>
      <Toaster richColors />
      <ContextMenu>
        <ContextMenuTrigger className="block min-h-screen w-full">
          <Outlet />
        </ContextMenuTrigger>

        <ContextMenuContent className="w-52">
          <ContextMenuItem inset disabled={!canGoBack} onClick={goBack}>
            Back
            <ContextMenuShortcut>⌘[</ContextMenuShortcut>
          </ContextMenuItem>
          <ContextMenuItem inset disabled={!canGoForward} onClick={goForward}>
            Forward
            <ContextMenuShortcut>⌘]</ContextMenuShortcut>
          </ContextMenuItem>
          <ContextMenuItem inset onClick={reloadWindow}>
            Reload
            <ContextMenuShortcut>⌘R</ContextMenuShortcut>
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    </>
  );
}

export default App;
