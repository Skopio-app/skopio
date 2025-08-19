import { useEffect, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
  Goal,
  TimeRangePreset,
  TimeSpan,
} from "../../../types/tauri.gen";
import BarLineChart from "./BarLineChart";
import GoalChartCard from "./GoalChartCard";
import GoalTitleDialog from "./Dialogs/GoalTitleDialog";
import GoalDeleteConfirmDialog from "./Dialogs/GoalDeleteConfirmDialog";
import GoalDialog from "./Dialogs/GoalDialog";

const GoalDisplay = ({ goal }: { goal: Goal }) => {
  const [data, setData] = useState<BucketTimeSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [showEditNameDialog, setShowEditNameDialog] = useState<boolean>(false);
  const [showGoalDeleteDialog, setShowGoalDeleteDialog] =
    useState<boolean>(false);
  const [showGoalDialog, setShowGoalDialog] = useState<boolean>(false);

  const timeRangeToPreset = (span: TimeSpan): TimeRangePreset => {
    switch (span) {
      case "day":
        return { lastNDays: [7, true] };
      case "week":
        return { lastNWeeks: [7, true] };
      case "month":
        return { lastNMonths: [7, true] };
      case "year":
        return { lastNYears: [7, true] };
      default:
        return { lastNDays: [7, true] };
    }
  };

  const query: BucketedSummaryInput = {
    preset: timeRangeToPreset(goal.timeSpan),
    apps: goal.useApps ? goal.apps : null,
    categories: goal.useCategories ? goal.categories : null,
  };

  useEffect(() => {
    const fetchData = async () => {
      try {
        const summary = await commands.fetchBucketedSummary(query);
        setData(summary);
      } catch (e) {
        console.error("Error fetching summary for goal: ", goal.id, e);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [goal.id]);

  const chartData = data.map((item) => ({
    label: item.bucket,
    value: item.grouped_values["Total"] ?? 0,
  }));

  return (
    <GoalChartCard
      title={goal.name}
      loading={loading}
      onRename={() => setShowEditNameDialog(true)}
      onDelete={() => setShowGoalDeleteDialog(true)}
      onEdit={() => setShowGoalDialog(true)}
    >
      <BarLineChart
        data={chartData}
        goalDuration={goal.targetSeconds}
        timeSpan={goal.timeSpan}
      />
      <GoalTitleDialog
        open={showEditNameDialog}
        onOpenChange={setShowEditNameDialog}
        goalId={goal.id}
        title={goal.name}
      />
      <GoalDeleteConfirmDialog
        open={showGoalDeleteDialog}
        onOpenChange={setShowGoalDeleteDialog}
        goalName={goal.name}
        goalId={goal.id}
      />
      <GoalDialog
        open={showGoalDialog}
        onOpenChange={setShowGoalDialog}
        goal={goal}
      />
    </GoalChartCard>
  );
};

export default GoalDisplay;
