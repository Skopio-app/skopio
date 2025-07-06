import { Button } from "@skopio/ui";
import { useEffect, useState } from "react";
import GoalDialog from "./GoalDialog";
import { commands, Goal } from "../../types/tauri.gen";
import GoalDisplay from "./components/GoalDisplay";

const GoalsView = () => {
  const [showGoalDialog, setShowGoalDialog] = useState<boolean>(false);
  const [goals, setGoals] = useState<Goal[]>([]);

  const fetchGoals = async () => {
    try {
      const data = await commands.getGoals();
      setGoals(data);
    } catch (err) {
      console.error("Failed to fetch goals: ", err);
    }
  };

  useEffect(() => {
    fetchGoals();
  }, [showGoalDialog]);

  return (
    <div className="w-full px-4 py-6 space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-lg font-semibold">Goals</h1>
        <Button variant="default" onClick={() => setShowGoalDialog(true)}>
          New Goal
        </Button>
      </div>

      <div className="flex flex-col space-y-4">
        {goals.map((goal) => (
          <GoalDisplay key={goal.id} goal={goal} />
        ))}
      </div>

      <GoalDialog open={showGoalDialog} onOpenChange={setShowGoalDialog} />
    </div>
  );
};

export default GoalsView;
