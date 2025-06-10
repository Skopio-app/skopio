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

const DashboardLayout = () => {
  const navigate = useNavigate();
  const location = useLocation();

  const tabExtensions = [...builtinExtensionRegistry.getTabExtensions()];

  return (
    <SidebarProvider className="bg-muted relative">
      <div
        className={cn(
          "fixed left-0 top-0 z-40 flex h-[20px] w-full items-center",
          "bg-background select-none px-4",
        )}
      >
        <div className="w-[64px] h-full" />

        <div className="absolute left-[76px] top-[4px]">
          <SidebarTrigger className="w-5 h-5 cursor-pointer" />
        </div>
      </div>

      <Sidebar collapsible="offcanvas" className="pt-[10px] bg-transparent">
        <SidebarHeader />
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupLabel>Builtin Extensions</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {tabExtensions.map((ext) => (
                  <SidebarMenuItem key={ext.manifest.id}>
                    <SidebarMenuButton
                      isActive={location.pathname === "/tab/" + ext.manifest.id}
                      onClick={() => navigate("/tab/" + ext.manifest.id)}
                    >
                      {ext.manifest.name}
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
