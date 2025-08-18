import {
  Button,
  cn,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@skopio/ui";
import { GripVertical, Settings, X } from "lucide-react";
import SkeletonChart from "../../../components/SkeletonChart";
import React from "react";

export interface WidgetCardProps {
  title?: string;
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
          "relative flex flex-col rounded-2xl border border-muted bg-white shadow-sm",
          "transition-shadow hover:shadow-md p-4",
          className,
        )}
      >
        <div className="flex items-center justify-between mb-2">
          {title && (
            <h2 className="text-sm font-medium text-muted-foreground">
              {title}
            </h2>
          )}
          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="icon"
              className="h-6 w-6 text-gray-500 hover:text-gray-700 cursor-grab"
              id={draggableHandleId}
              aria-label="Drag"
            >
              <GripVertical className="h-4 w-4" />
            </Button>
            {settingsContent && (
              <Popover onOpenChange={onSettingsOpenChange}>
                <PopoverTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 text-gray-500 hover:text-gray-700"
                    aria-label="Settings"
                  >
                    <Settings className="h-4 w-4" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent
                  align="end"
                  sideOffset={8}
                  className="w-auto rounded-lg border bg-white p-4 shadow-xl"
                >
                  {settingsContent}
                </PopoverContent>
              </Popover>
            )}
            {onRemove && (
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6 text-gray-500 hover:text-red-500"
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
