import { useMemo } from "react";
import WidgetCard from "../components/WidgetCard";
import { useSummaryData } from "../hooks/useSummaryData";
import CustomPieChart from "../../../components/CustomPieChart";
import { PieChartData } from "../../../types/types";

const AppPieChartWidget = () => {
  const { getGroupData, loading } = useSummaryData(undefined, ["app"]);

  const data = useMemo<PieChartData[]>(() => {
    const totals = getGroupData("app");
    return Object.entries(totals).map(([id, value]) => ({
      id,
      label: id,
      value,
    }));
  }, [getGroupData]);

  return (
    <WidgetCard title="Apps" loading={loading}>
      <CustomPieChart data={data} />
    </WidgetCard>
  );
};

export default AppPieChartWidget;
