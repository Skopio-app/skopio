import { Button } from "@skopio/ui";
import { useEffect, useState } from "react";
import GoalDialog from "./components/Dialogs/GoalDialog";
import GoalDisplay from "./components/GoalDisplay";
import { useGoalStore } from "./stores/useGoalStore";
import { Plus } from "lucide-react";

const GoalsView = () => {
  const [showGoalDialog, setShowGoalDialog] = useState<boolean>(false);
  const { goals, loading, fetchGoals } = useGoalStore();

  useEffect(() => {
    fetchGoals();
  }, []);

  if (loading) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-muted-foreground animate-pulse">
        Loading...
      </div>
    );
  }

  return (
    <div className="w-full px-4 space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-lg text-foreground font-semibold">Goals</h1>
        <Button variant="secondary" onClick={() => setShowGoalDialog(true)}>
          <Plus /> New Goal
        </Button>
      </div>

      <div className="flex flex-col space-y-4 pl-3">
        {goals.length > 0 ? (
          goals.map((goal) => <GoalDisplay key={goal.id} goal={goal} />)
        ) : (
          <p className="h-64 w-full flex items-center justify-center text-sm text-gray-500">
            No goals found
          </p>
        )}
      </div>

      <GoalDialog open={showGoalDialog} onOpenChange={setShowGoalDialog} />
    </div>
  );
};

export default GoalsView;
