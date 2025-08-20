import {
  cn,
  Sidebar,
  SidebarContent,
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
import { builtinExtensionRegistry } from "../extensions/registry";
import { LAST_ACTIVE_TAB } from "../utils/constants";
import { Cog } from "lucide-react";

const DashboardLayout = () => {
  const navigate = useNavigate();
  const location = useLocation();

  const tabExtensions = [...builtinExtensionRegistry.getTabExtensions()];

  return (
    <SidebarProvider className="bg-muted relative">
      <div
        data-tauri-drag-region
        className={cn(
          "fixed left-0 top-0 z-40 flex h-[20px] w-full items-center",
          "bg-background select-none px-4",
        )}
      >
        <div className="w-[64px] h-full" />

        <div className="absolute left-[90px] top-[6px] flex flex-row space-x-3">
          <SidebarTrigger className="w-5 h-5 cursor-pointer hover:bg-neutral-200" />
          <Cog className="h-5 w-5 p-0.5 cursor-pointer hover:bg-neutral-200" />
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
      </Sidebar>

      <SidebarInset>
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
};

export default DashboardLayout;
