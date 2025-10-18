import {
  cn,
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuShortcut,
  ContextMenuTrigger,
} from "@skopio/ui";
import { Outlet } from "react-router";
import { Toaster } from "sonner";
import {
  keepPreviousData,
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
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
import UpdaterToast from "@/components/updater/UpdaterToast";
import ThemeProvider from "@/components/settings/ThemeProvider";
// import DinoLoading from "./components/DinoLoading";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      networkMode: "always",
      retry: 2,
      staleTime: 60_000,
      placeholderData: keepPreviousData,
    },
    mutations: {
      networkMode: "always",
    },
  },
});

function App() {
  useGlobalShortcutListener();
  const { canGoBack, canGoForward } = useHistoryControls();

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <Toaster richColors />
        <PermissionsDialog />
        <UpdaterToast />
        <ContextMenu>
          <ContextMenuTrigger className="block min-h-screen w-full">
            <Outlet />
          </ContextMenuTrigger>

          <ContextMenuContent className={cn("w-42", isDev() ? "h-32" : "h-24")}>
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
              <ContextMenuShortcut>⌘R</ContextMenuShortcut>
            </ContextMenuItem>
            {isDev() && (
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
      </ThemeProvider>
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
    // <DinoLoading />
  );
}

export default App;
