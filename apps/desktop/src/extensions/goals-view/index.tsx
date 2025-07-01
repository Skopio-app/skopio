import { Button } from "@skopio/ui";
import { useState } from "react";
import GoalDialog from "./GoalDialog";

const GoalsView = () => {
  const [showGoalDialog, setShowGoalDialog] = useState<boolean>(false);
  return (
    <div className="flex items-center">
      <Button
        variant="default"
        className="hover:cursor-pointer"
        onClick={() => setShowGoalDialog(true)}
      >
        New goal
      </Button>

      <GoalDialog open={showGoalDialog} onOpenChange={setShowGoalDialog} />
    </div>
  );
};

export default GoalsView;
