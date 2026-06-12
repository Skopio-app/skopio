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
import { useEffect } from "react";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import {
  goBack,
  goForward,
  reloadWindow,
  useGlobalShortcutListener,
  useHistoryControls,
} from "@/utils/shortcut";
import PermissionsDialog from "@/components/settings/PermissionsDialog";
import UpdaterToast from "@/components/updater/UpdaterToast";
import ThemeProvider from "@/components/settings/ThemeProvider";
import { TourProvider } from "./utils/tour/TourProvider";

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
  useDisableNativeContextMenu();
  const { canGoBack, canGoForward } = useHistoryControls();

  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <TourProvider>
          <Toaster richColors />
          <PermissionsDialog />
          <UpdaterToast />
          <ContextMenu>
            <ContextMenuTrigger className="block min-h-screen w-full">
              <Outlet />
            </ContextMenuTrigger>

            <ContextMenuContent className={cn("w-42", "h-24")}>
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
            </ContextMenuContent>
          </ContextMenu>
        </TourProvider>
      </ThemeProvider>
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}

const useDisableNativeContextMenu = () => {
  useEffect(() => {
    const preventNativeContextMenu = (event: MouseEvent) => {
      event.preventDefault();
    };

    document.addEventListener("contextmenu", preventNativeContextMenu);
    return () => {
      document.removeEventListener("contextmenu", preventNativeContextMenu);
    };
  }, []);
};

export default App;
