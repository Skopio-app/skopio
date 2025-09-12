import {
  Button,
  cn,
  Separator,
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarTrigger,
} from "@skopio/ui";
import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { builtinExtensionRegistry } from "@/extensions/registry";
import { LAST_ACTIVE_TAB } from "@/utils/constants";
import { Cog } from "lucide-react";
import { commands, ServerStatus } from "@/types/tauri.gen";
import { useServerStatus } from "@/hooks/useServerStatus";
import { isDev } from "@/utils/environment";

const DashboardLayout = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const status = useServerStatus();

  const tabExtensions = [...builtinExtensionRegistry.getTabExtensions()];

  const renderStatus = (s: ServerStatus): string => {
    switch (s.state) {
      case "running":
        return "Running";
      case "starting":
        return "Starting...";
      case "checking":
        return "Checking...";
      case "installing":
        return "Installing";
      case "updating":
        return "Updating...";
      case "downloading": {
        const pct = s.percent ?? 0;
        return `Downloading... ${pct}%`;
      }
      case "error":
        return `Error: ${s.message}`;
      default:
        return "Offline";
    }
  };

  return (
    <SidebarProvider className="bg-muted relative">
      <div
        data-tauri-drag-region
        className={cn(
          "fixed left-0 top-0 z-40 flex h-[20px] w-full items-center",
          "bg-transparent select-none px-4",
        )}
      >
        <div className="w-[64px] h-full" />

        <div className="absolute left-[90px] top-[6px] flex flex-row space-x-3">
          <SidebarTrigger className="w-5 h-5 cursor-pointer hover:bg-neutral-200" />
          <Button
            className="h-5 w-5 hover:bg-neutral-200"
            variant="ghost"
            size="icon"
            onClick={async () => await commands.showWindow("settings")}
          >
            <Cog />
          </Button>
        </div>
      </div>

      <Sidebar
        collapsible="offcanvas"
        className="pt-[10px] border-border bg-neutral-50"
      >
        <SidebarHeader />
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupLabel>Builtin Extensions</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {tabExtensions.map((ext) => (
                  <SidebarMenuItem key={ext.manifest.id}>
                    <SidebarMenuButton
                      tooltip={{
                        children: ext.manifest.description,
                        hidden: true,
                      }}
                      isActive={location.pathname === "/tab/" + ext.manifest.id}
                      onClick={() => {
                        localStorage.setItem(LAST_ACTIVE_TAB, ext.manifest.id);
                        navigate("/tab/" + ext.manifest.id);
                      }}
                    >
                      <ext.manifest.icon />
                      <span>{ext.manifest.name}</span>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>
        {!isDev() && (
          <SidebarFooter className="space-y-1">
            <Separator />
            <p className="text-xs text-neutral-500 flex items-center gap-2">
              <span
                className={cn(
                  "inline-block h-2 w-2 rounded-full",
                  status.state === "running"
                    ? "bg-green-500 animate-pulse"
                    : status.state === "offline" || status.state === "error"
                      ? "bg-red-500"
                      : "bg-yellow-500 animate-pulse",
                )}
              />
              Server status: {renderStatus(status)}
            </p>

            {status.state === "downloading" && (
              <div className="mt-1 h-1 w-full bg-neutral-200 rounded">
                <div
                  className="h-1 bg-neutral-500 rounded"
                  style={{ width: `${status.percent ?? 0}%` }}
                />
              </div>
            )}
          </SidebarFooter>
        )}
      </Sidebar>

      <SidebarInset className="py-4">
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
};

export default DashboardLayout;
