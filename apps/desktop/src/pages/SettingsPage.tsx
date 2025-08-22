import { cn } from "@skopio/ui";
import {
  Calendar,
  LucideIcon,
  MonitorCog,
  Settings,
  Settings2,
} from "lucide-react";
import { NavLink, Outlet, useLocation } from "react-router";

type Tab = "general" | "permissions";

const TABS: { name: Tab; icon: LucideIcon }[] = [
  { name: "general", icon: Settings },
  { name: "permissions", icon: Calendar },
];

export function TabIcon({ tab }: { tab: Tab }) {
  switch (tab) {
    case "general":
      return <Settings2 className="h-4 w-4" />;
    case "permissions":
      return <MonitorCog className="h-4 w-4" />;
    default:
      return null;
  }
}

const SettingsPage = () => {
  const { pathname } = useLocation();
  const activeTab = (pathname.split("/").pop() as Tab | undefined) ?? "general";

  const getTabTitle = (tab: Tab) =>
    tab === "general" ? "General" : tab === "permissions" ? "Permissions" : tab;

  return (
    <div className="flex min-h-dvh w-full items-stretch overflow-hidden">
      <aside className="flex w-60 flex-col border-r">
        <div
          data-tauri-drag-region
          className="flex h-11 items-center justify-end px-2"
        />
        <div className="min-h-0 flex-1 overflow-y-auto p-2">
          <nav className="space-y-1">
            {TABS.map((tab) => (
              <NavLink
                key={tab.name}
                to={tab.name}
                className={({ isActive }) =>
                  cn(
                    "flex w-full items-center gap-2 rounded-lg p-2 text-sm text-neutral-600 hover:bg-neutral-100",
                    isActive && "bg-neutral-100 font-medium",
                  )
                }
                end
              >
                <TabIcon tab={tab.name} />
                <span className="text-neutral-900 capitalize">
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
          className="flex h-11 w-full items-center justify-between border-b px-2"
        >
          <div className="w-40" data-tauri-drag-region />
          <h1
            className="text-md font-semibold capitalize"
            data-tauri-drag-region
          >
            {getTabTitle(activeTab)}
          </h1>
          <div className="w-40" data-tauri-drag-region />
        </header>

        <div className="min-h-0 flex-1 overflow-y-auto p-6 w-full">
          <Outlet />
        </div>
      </main>
    </div>
  );
};

export default SettingsPage;
