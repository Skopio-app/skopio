import { useEffect, useState } from "react";
import ChartContainer from "./ChartContainer";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
} from "../../../types/tauri.gen";
import LineChart from "./LineChart";
import { usePresetFilter } from "../stores/usePresetFilter";

interface LineChartSectionProps {
  projectName: string;
}

const LineChartSection: React.FC<LineChartSectionProps> = ({ projectName }) => {
  const [loading, setLoading] = useState(true);
  const [data, setData] = useState<BucketTimeSummary[]>([]);

  const { preset } = usePresetFilter();

  const query: BucketedSummaryInput = {
    preset: preset,
    project_names: [projectName],
    include_afk: false,
  };

  const fetchData = async () => {
    try {
      const summary = await commands.fetchBucketedSummary(query);
      setData(summary);
    } catch (err) {
      console.error("Error fetching line chart summary data: ", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, [projectName, preset]);

  const chartData = data.map((item) => ({
    x: item.bucket,
    y: item.grouped_values["Total"] ?? 0,
  }));

  return (
    <ChartContainer loading={loading}>
      <LineChart id="Duration" data={chartData} />
    </ChartContainer>
  );
};

export default LineChartSection;
