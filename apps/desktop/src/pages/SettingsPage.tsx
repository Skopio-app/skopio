import { cn } from "@skopio/ui";
import { LucideIcon, MonitorCog, Settings2 } from "lucide-react";
import { NavLink, Outlet, useLocation } from "react-router";

type Tab = "general" | "permissions";

const TABS: { name: Tab; icon: LucideIcon }[] = [
  { name: "general", icon: Settings2 },
  { name: "permissions", icon: MonitorCog },
];

const SettingsPage = () => {
  const { pathname } = useLocation();
  const activeTab = (pathname.split("/").pop() as Tab | undefined) ?? "general";

  const getTabTitle = (tab: Tab) =>
    tab === "general" ? "General" : tab === "permissions" ? "Permissions" : tab;

  return (
    <div className="bg-sidebar fixed inset-0 flex w-full items-stretch overflow-hidden">
      <aside className="flex w-60 flex-none flex-col border-r overflow-hidden">
        <div
          data-tauri-drag-region
          className="flex h-11 items-center justify-end px-2"
        />
        <div className="p-2">
          <nav className="space-y-1">
            {TABS.map((tab) => (
              <NavLink
                key={tab.name}
                to={tab.name}
                className={({ isActive }) =>
                  cn(
                    "flex w-full items-center gap-2 rounded-lg p-2 text-sm text-muted-foreground hover:bg-muted",
                    isActive && "bg-border font-medium",
                  )
                }
                end
              >
                <tab.icon />
                <span className="text-foreground capitalize">
                  {getTabTitle(tab.name)}
                </span>
              </NavLink>
            ))}
          </nav>
        </div>
      </aside>

      <main className="flex min-h-0 flex-1 flex-col overflow-hidden">
        <header
          data-tauri-drag-region
          className="flex h-11 w-full flex-none items-center justify-between border-b px-2"
        >
          <div className="w-40" data-tauri-drag-region />
          <h1
            className="text-md font-semibold text-foreground capitalize"
            data-tauri-drag-region
          >
            {getTabTitle(activeTab)}
          </h1>
          <div className="w-40" data-tauri-drag-region />
        </header>

        <div className="min-h-0 min-w-0 flex-1 overflow-auto p-6">
          <Outlet />
        </div>
      </main>
    </div>
  );
};

export default SettingsPage;
