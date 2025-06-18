import ProjectBarChart from "../ProjectBarChart";
import WidgetCard from "../WidgetCard";

const mockData = [
  {
    week: "Week 1",
    ProjectX: 12,
    ProjectY: 5,
    ProjectZ: 3,
  },
  {
    week: "Week 2",
    ProjectX: 8,
    ProjectY: 9,
    ProjectZ: 6,
  },
  {
    week: "Week 3",
    ProjectX: 10,
    ProjectY: 4,
    ProjectZ: 8,
  },
  {
    week: "Week 4",
    ProjectX: 6,
    ProjectY: 7,
    ProjectZ: 5,
  },
];

const ProjectChartWidget = () => {
  return (
    <WidgetCard title="Weekly project time" onRemove={() => {}}>
      <ProjectBarChart
        data={mockData}
        keys={["ProjectX", "ProjectY", "ProjectZ"]}
      />
    </WidgetCard>
  );
};

export default ProjectChartWidget;
