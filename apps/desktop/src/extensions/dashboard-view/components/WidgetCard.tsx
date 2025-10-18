import {
  Button,
  cn,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@skopio/ui";
import { GripVertical, Info, Settings, X } from "lucide-react";
import SkeletonChart from "@/components/loading/SkeletonChart";
import React from "react";

export interface WidgetCardProps {
  title?: string;
  tooltip?: string;
  onSettingsOpenChange?: (open: boolean) => void;
  onRemove?: () => void;
  children: React.ReactNode;
  className?: string;
  draggableHandleId?: string;
  loading: boolean;
  settingsContent?: React.ReactNode;
}

const WidgetCard = React.forwardRef<HTMLDivElement, WidgetCardProps>(
  (
    {
      title,
      tooltip,
      onRemove,
      onSettingsOpenChange,
      children,
      className,
      draggableHandleId = "widget-drag-handle",
      loading,
      settingsContent,
    },
    ref,
  ) => {
    return (
      <div
        ref={ref}
        className={cn(
          "relative flex flex-col rounded-2xl border border-muted bg-background shadow-sm",
          "transition-shadow hover:shadow-md p-4",
          className,
        )}
      >
        <div className="flex items-center justify-between mb-2">
          {title && (
            <h2 className="text-sm font-medium text-foreground">{title}</h2>
          )}
          <div className="flex items-center gap-1">
            {tooltip && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <Info className="h-4 w-4 mr-2 cursor-pointer text-muted-foreground" />
                </TooltipTrigger>
                <TooltipContent>{tooltip}</TooltipContent>
              </Tooltip>
            )}
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6 cursor-grab"
              id={draggableHandleId}
              aria-label="Drag"
            >
              <GripVertical className="h-4 w-4 text-muted-foreground" />
            </Button>
            {settingsContent && (
              <Popover onOpenChange={onSettingsOpenChange}>
                <PopoverTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 text-muted-foreground"
                    aria-label="Settings"
                  >
                    <Settings className="h-4 w-4" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent
                  align="end"
                  sideOffset={8}
                  className="w-auto rounded-lg border p-4 shadow-xl"
                >
                  {settingsContent}
                </PopoverContent>
              </Popover>
            )}
            {onRemove && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6 text-muted-foreground hover:text-destructive"
                onClick={onRemove}
                aria-label="Close"
              >
                <X className="h-4 w-4" />
              </Button>
            )}
          </div>
        </div>

        {loading ? (
          <SkeletonChart />
        ) : (
          <div className="flex-1 overflow-hidden">{children}</div>
        )}
      </div>
    );
  },
);

WidgetCard.displayName = "WidgetCard";

export default WidgetCard;
