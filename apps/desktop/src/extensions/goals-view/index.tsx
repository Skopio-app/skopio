import { Button } from "@skopio/ui";
import { useEffect, useState } from "react";
import GoalDialog from "./GoalDialog";
import GoalDisplay from "./components/GoalDisplay";
import { useGoalStore } from "./stores/useGoalStore";

const GoalsView = () => {
  const [showGoalDialog, setShowGoalDialog] = useState<boolean>(false);
  const { goals, loading, fetchGoals } = useGoalStore();

  useEffect(() => {
    fetchGoals();
  }, []);

  if (!loading && goals.length === 0) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-gray-500">
        No goals found
      </div>
    );
  }

  return (
    <div className="w-full px-4 py-6 space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-lg font-semibold">Goals</h1>
        <Button variant="default" onClick={() => setShowGoalDialog(true)}>
          New Goal
        </Button>
      </div>

      <div className="flex flex-col space-y-4 pl-3">
        {goals.map((goal) => (
          <GoalDisplay key={goal.id} goal={goal} />
        ))}
      </div>

      <GoalDialog open={showGoalDialog} onOpenChange={setShowGoalDialog} />
    </div>
  );
};

export default GoalsView;
