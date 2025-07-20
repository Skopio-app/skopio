import { useMemo } from "react";
import { PieChartData } from "../../../types/types";
import WidgetCard from "../components/WidgetCard";
import CustomPieChart from "../charts/CustomPieChart";
import { useSummaryData } from "../hooks/useSummaryData";

const LanguagePieChartWidget = () => {
  const { getGroupData, loading } = useSummaryData(undefined, ["language"]);

  const data = useMemo<PieChartData[]>(() => {
    const totals = getGroupData("language");
    return Object.entries(totals).map(([id, value]) => ({
      id,
      label: id,
      value,
    }));
  }, [getGroupData]);

  return (
    <WidgetCard title="Languages" loading={loading}>
      <CustomPieChart data={data} />
    </WidgetCard>
  );
};

export default LanguagePieChartWidget;
