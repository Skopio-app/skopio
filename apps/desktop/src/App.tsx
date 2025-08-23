// import {
//   ContextMenu,
//   ContextMenuContent,
//   ContextMenuItem,
//   ContextMenuSeparator,
//   ContextMenuShortcut,
//   ContextMenuTrigger,
// } from "@skopio/ui";
import { Outlet } from "react-router";
import { Toaster } from "sonner";
import { useGlobalShortcutListener } from "./utils/shortcut";

function App() {
  useGlobalShortcutListener();

  return (
    <>
      <Toaster richColors />
      {/* <ContextMenu>
        <ContextMenuTrigger className="block min-h-screen w-full"> */}
      <Outlet />
      {/* </ContextMenuTrigger>
        <ContextMenuContent className="w-52">
          <ContextMenuItem inset>
            Back
            <ContextMenuShortcut>⌘[</ContextMenuShortcut>
          </ContextMenuItem>
          <ContextMenuItem inset disabled>
            Forward
            <ContextMenuShortcut>⌘]</ContextMenuShortcut>
          </ContextMenuItem>
          <ContextMenuItem inset>
            Reload
            <ContextMenuShortcut>⌘R</ContextMenuShortcut>
          </ContextMenuItem>
          <ContextMenuSeparator />
        </ContextMenuContent>
      </ContextMenu> */}
    </>
  );
}

export default App;
