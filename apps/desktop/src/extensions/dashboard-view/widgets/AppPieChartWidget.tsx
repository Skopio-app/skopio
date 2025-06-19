import AppPieChart from "../charts/AppPieChart";
import WidgetCard from "../WidgetCard";

const mockData = [
  {
    id: "Code",
    label: "Code",
    value: 20,
    color: "hsl(333, 70%, 50%)",
  },
  {
    id: "desktop",
    label: "desktop",
    value: 35,
    color: "hsl(352, 70%, 50%)",
  },
  {
    id: "iPhone Mirroring",
    label: "iPhone Mirroring",
    value: 15,
    color: "hsl(240, 70%, 50%)",
  },
  {
    id: "Activity Monitor",
    label: "Activity Monitor",
    value: 62,
    color: "hsl(202, 70%, 50%)",
  },
];

const AppPieChartWidget = () => {
  return (
    <WidgetCard title="Apps" onRemove={() => {}}>
      <AppPieChart data={mockData} />
    </WidgetCard>
  );
};

export default AppPieChartWidget;
