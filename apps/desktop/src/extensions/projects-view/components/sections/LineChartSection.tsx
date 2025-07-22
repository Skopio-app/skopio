import ChartContainer from "../ChartContainer";
import LineChart from "../LineChart";
import { useProjectSummaryData } from "../../hooks/useProjectSummaryData";

const LineChartSection = () => {
  const options = { mode: "line" as const };
  const { data, loading } = useProjectSummaryData(options);

  return (
    <ChartContainer loading={loading}>
      <LineChart id="Duration" data={data} />
    </ChartContainer>
  );
};

export default LineChartSection;
