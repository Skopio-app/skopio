import { cn } from "@skopio/ui";
import { Clock, Pencil, Settings, Trash2, Type } from "lucide-react";
import React from "react";
import SkeletonChart from "../../../components/SkeletonChart";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";

interface GoalChartCardProps {
  title?: string;
  children: React.ReactNode;
  className?: string;
  loading: boolean;
  onRename?: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
  onSnooze?: () => void;
}

const SettingsButton = React.forwardRef<
  HTMLButtonElement,
  React.ButtonHTMLAttributes<HTMLButtonElement> & {
    variant?: string;
    size?: string;
  }
>(({ className, children, variant, size, ...props }, ref) => {
  return (
    <button
      ref={ref}
      className={cn(
        "inline-flex items-center justify-center rounded-md px-3 py-1.5 text-sm font-medium transition-colors cursor-pointer",
        variant === "ghost" && "bg-transparent hover:bg-gray-100",
        size === "icon" && "p-2",
        className,
      )}
      {...props}
    >
      {children}
    </button>
  );
});

SettingsButton.displayName = "SettingsButton";

const GoalChartCard: React.FC<GoalChartCardProps> = ({
  title,
  children,
  className,
  loading,
  onRename,
  onEdit,
  onDelete,
  onSnooze,
}) => {
  return (
    <div
      className={cn(
        "relative flex flex-col rounded-2xl border border-muted bg-white shadow-sm w-full max-w-6xl",
        "transition-shadow hover:shadow-md p-4",
        className,
      )}
    >
      <div className="flex items-center justify-between mb-2">
        {title && (
          <h2 className="text-sm font-medium text-muted-foreground">{title}</h2>
        )}
        <div className="flex items-center gap-1">
          <DropdownMenu.Root>
            <DropdownMenu.Trigger asChild>
              <SettingsButton variant="ghost" size="icon">
                <Settings className="h-4 w-4" />
              </SettingsButton>
            </DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                side="bottom"
                align="end"
                className="z-50 min-w-[160px] rounded-md bg-white shadow-lg border border-gray-200 p-1"
              >
                <DropdownMenu.Item
                  onSelect={onRename}
                  className="cursor-pointer flex items-center gap-2 rounded px-3 py-2 hover:bg-gray-100"
                >
                  <Type className="w-4 h-4 text-gray-500" />
                  Rename
                </DropdownMenu.Item>
                <DropdownMenu.Item
                  onSelect={onEdit}
                  className="cursor-pointer flex items-center gap-2 rounded px-3 py-2 hover:bg-gray-100"
                >
                  <Pencil className="w-4 h-4 text-gray-500" />
                  Edit
                </DropdownMenu.Item>
                <DropdownMenu.Item
                  onSelect={onSnooze}
                  className="cursor-pointer flex items-center gap-2 rounded px-3 py-2 hover:bg-gray-100"
                >
                  <Clock className="w-4 h-4 text-gray-500" />
                  Snooze
                </DropdownMenu.Item>
                <DropdownMenu.Separator className="m-2 h-px bg-neutral-400" />
                <DropdownMenu.Item
                  onSelect={onDelete}
                  className="cursor-pointer flex items-center gap-2 rounded px-3 py-2 text-red-600 hover:bg-gray-100"
                >
                  <Trash2 className="w-4 h-4 text-red-600" />
                  Delete
                </DropdownMenu.Item>
              </DropdownMenu.Content>
            </DropdownMenu.Portal>
          </DropdownMenu.Root>
        </div>
      </div>

      {loading ? (
        <SkeletonChart />
      ) : (
        <div className="flex-1 overflow-hidden">{children}</div>
      )}
    </div>
  );
};

export default GoalChartCard;
