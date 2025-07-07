import { Button, cn } from "@skopio/ui";
import { Settings } from "lucide-react";
import React from "react";
import SkeletonChart from "../../../components/SkeletonChart";

interface GoalChartCardProps {
  title?: string;
  children: React.ReactNode;
  className?: string;
  loading: boolean;
}

const GoalChartCard: React.FC<GoalChartCardProps> = ({
  title,
  children,
  className,
  loading,
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
          <Button
            variant="ghost"
            size="icon"
            className="h-6 w-6 text-gray-500 hover:text-gray-700"
          >
            <Settings />
          </Button>
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
