import { cn } from "@skopio/ui";
import { Pencil, Settings, Trash2, Type } from "lucide-react";
import React from "react";
import SkeletonChart from "@/components/loading/SkeletonChart";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";

interface GoalChartCardProps {
  title?: string;
  children: React.ReactNode;
  className?: string;
  loading: boolean;
  onRename?: () => void;
  onEdit?: () => void;
  onDelete?: () => void;
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
        variant === "ghost" && "bg-transparent hover:bg-accent",
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
}) => {
  return (
    <div
      className={cn(
        "relative flex flex-col rounded-2xl border border-muted shadow-sm w-full",
        "transition-shadow hover:shadow-md p-4",
        className,
      )}
    >
      <div className="flex items-center justify-between mb-2">
        {title && (
          <h2 className="text-sm text-foreground font-medium">{title}</h2>
        )}
        <div className="flex items-center gap-1">
          <DropdownMenu.Root>
            <DropdownMenu.Trigger asChild>
              <SettingsButton
                variant="ghost"
                size="icon"
                className="text-foreground"
              >
                <Settings className="h-4 w-4" />
              </SettingsButton>
            </DropdownMenu.Trigger>
            <DropdownMenu.Portal>
              <DropdownMenu.Content
                side="bottom"
                align="end"
                className="z-50 min-w-[160px] rounded-md bg-background shadow-lg border border-border p-1"
              >
                <DropdownMenu.Item
                  onSelect={onRename}
                  className="cursor-pointer flex items-center gap-2 rounded px-3 py-2 hover:bg-accent text-muted-foreground"
                >
                  <Type className="w-4 h-4 text-muted-foreground" />
                  Rename
                </DropdownMenu.Item>
                <DropdownMenu.Item
                  onSelect={onEdit}
                  className="cursor-pointer flex items-center gap-2 rounded px-3 py-2 hover:bg-accent text-muted-foreground"
                >
                  <Pencil className="w-4 h-4 text-muted-foreground" />
                  Edit
                </DropdownMenu.Item>
                <DropdownMenu.Separator className="m-2 h-px bg-[var(--muted)]" />
                <DropdownMenu.Item
                  onSelect={onDelete}
                  className="cursor-pointer flex items-center gap-2 rounded px-3 py-2 text-destructive hover:bg-accent text-muted-foreground"
                >
                  <Trash2 className="w-4 h-4 text-destructive" />
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
