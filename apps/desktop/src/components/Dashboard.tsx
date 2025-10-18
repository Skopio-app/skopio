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
  SidebarMenuSkeleton,
  SidebarProvider,
  SidebarTrigger,
} from "@skopio/ui";
import { Outlet, useLocation, useNavigate } from "react-router";
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
    <SidebarProvider className="bg-muted fixed inset-0 overflow-hidden">
      <div
        data-tauri-drag-region
        className={cn(
          "fixed left-0 top-0 z-40 flex h-9 w-full items-center",
          "bg-background border border-none shadow-sm select-none px-4",
        )}
      >
        <div className="w-16 h-full" />

        <div className="absolute left-23 top-1.5 flex flex-row space-x-3">
          <SidebarTrigger className="w-5 h-5 cursor-pointer hover:bg-background" />
          <Button
            className="h-5 w-5 text-foreground hover:bg-background"
            variant="ghost"
            size="icon"
            onClick={async () => await commands.showWindow("settings")}
          >
            <Cog />
          </Button>
        </div>
      </div>

      <Sidebar collapsible="offcanvas" className="pt-4 border-border shadow-md">
        <SidebarHeader />
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupLabel>Sections</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {tabExtensions.map((ext) => (
                  <SidebarMenuItem key={ext.manifest.id}>
                    {status.state === "running" ? (
                      <SidebarMenuButton
                        tooltip={{
                          children: ext.manifest.description,
                          hidden: true,
                        }}
                        isActive={
                          location.pathname === "/tab/" + ext.manifest.id
                        }
                        onClick={() => {
                          localStorage.setItem(
                            LAST_ACTIVE_TAB,
                            ext.manifest.id,
                          );
                          navigate("/tab/" + ext.manifest.id);
                        }}
                      >
                        <ext.manifest.icon />
                        <span>{ext.manifest.name}</span>
                      </SidebarMenuButton>
                    ) : (
                      <SidebarMenuSkeleton />
                    )}
                  </SidebarMenuItem>
                ))}
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>
        </SidebarContent>
        {!isDev() && (
          <SidebarFooter className="space-y-1">
            <Separator />
            <p className="text-xs text-muted-foreground flex items-center gap-2">
              <span
                className={cn(
                  "inline-block h-2 w-2 rounded-full",
                  status.state === "running"
                    ? "bg-green-500 animate-pulse"
                    : status.state === "offline" || status.state === "error"
                      ? "bg-destructive"
                      : "bg-yellow-500 animate-pulse",
                )}
              />
              Server status: {renderStatus(status)}
            </p>

            {status.state === "downloading" && (
              <div className="mt-1 h-1 w-full bg-muted rounded">
                <div
                  className="h-1 bg-muted-foreground rounded"
                  style={{ width: `${status.percent ?? 0}%` }}
                />
              </div>
            )}
          </SidebarFooter>
        )}
      </Sidebar>

      <SidebarInset className="pt-10 h-dvh overflow-auto overscroll-contain">
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
};

export default DashboardLayout;
