// import { Button } from "@skopio/ui";
// import { useState } from "react";
// import GoalDialog from "./GoalDialog";

// const GoalsView = () => {
//   const [showGoalDialog, setShowGoalDialog] = useState<boolean>(false);
//   return (
//     <div className="flex items-center">
//       <Button
//         variant="default"
//         className="hover:cursor-pointer"
//         onClick={() => setShowGoalDialog(true)}
//       >
//         New goal
//       </Button>

//       <GoalDialog open={showGoalDialog} onOpenChange={setShowGoalDialog} />
//     </div>
//   );
// };

import React from "react";
import BarLineChart from "./components/BarLineChart";
import GoalChartCard from "./components/GoalChartCard";

const TestChartContainer: React.FC = () => {
  const sampleDataMixed = [
    { label: "Mon", value: 300 }, // Below goal
    { label: "Tue", value: 450 }, // Above goal
    { label: "Wed", value: 250 }, // Below goal
    { label: "Thu", value: 500 }, // Above goal
    { label: "Fri", value: 380 }, // Below goal
    { label: "Sat", value: 600 }, // Well above goal
    { label: "Sun", value: 150 }, // Far below goal
  ];

  const goalMixed = 400;
  return (
    <div style={{ padding: "20px", maxWidth: "800px", margin: "auto" }}>
      <h2>Daily Active Time vs. Goal (Mixed Results)</h2>
      <BarLineChart data={sampleDataMixed} goalDuration={goalMixed} />

      <GoalChartCard title="All Below Goal" loading={false}>
        <BarLineChart
          data={[
            { label: "Q1", value: 100 },
            { label: "Q2", value: 180 },
            { label: "Q3", value: 90 },
            { label: "Q4", value: 220 },
          ]}
          goalDuration={300}
        />
      </GoalChartCard>

      <h2 style={{ marginTop: "40px" }}>All Above Goal</h2>
      <BarLineChart
        data={[
          { label: "Dept A", value: 700 },
          { label: "Dept B", value: 850 },
          { label: "Dept C", value: 720 },
          { label: "Dept D", value: 900 },
        ]}
        goalDuration={650}
      />

      <h2 style={{ marginTop: "40px" }}>Some Equal to Goal</h2>
      <BarLineChart
        data={[
          { label: "Task 1", value: 100 },
          { label: "Task 2", value: 150 }, // Equal to goal
          { label: "Task 3", value: 180 },
          { label: "Task 4", value: 150 }, // Equal to goal
          { label: "Task 5", value: 120 },
        ]}
        goalDuration={150}
      />

      <h2 style={{ marginTop: "40px" }}>Empty Data</h2>
      <BarLineChart data={[]} goalDuration={200} />

      <h2 style={{ marginTop: "40px" }}>Goal at Zero</h2>
      <BarLineChart
        data={[
          { label: "A", value: 10 },
          { label: "B", value: 20 },
        ]}
        goalDuration={0}
      />

      <h2 style={{ marginTop: "40px" }}>Data with Zero Values</h2>
      <BarLineChart
        data={[
          { label: "X", value: 0 },
          { label: "Y", value: 50 },
          { label: "Z", value: 0 },
        ]}
        goalDuration={30}
      />
    </div>
  );
};

export default TestChartContainer;
