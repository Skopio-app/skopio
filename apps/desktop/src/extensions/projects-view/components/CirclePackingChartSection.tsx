import { useEffect, useMemo, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
} from "../../../types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";
import ChartContainer from "./ChartContainer";
import CirclePackingChart from "./CirclePackingChart";

interface CirclePackingChartSectionProps {
  projectName: string;
}

const CirclePackingChartSection: React.FC<CirclePackingChartSectionProps> = ({
  projectName,
}) => {
  const [loading, setLoading] = useState(true);
  const [data, setData] = useState<BucketTimeSummary[]>([]);

  const { preset } = usePresetFilter();

  const query: BucketedSummaryInput = {
    preset,
    project_names: [projectName],
    group_by: "entity",
    include_afk: false,
  };

  const fetchData = async () => {
    try {
      const summary = await commands.fetchBucketedSummary(query);
      setData(summary);
    } catch (err) {
      console.error("Error fetching circle packing chart summary data: ", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, [projectName, preset]);

  const processedData = useMemo(() => {
    const merged: Record<string, number> = {};

    for (const item of data) {
      for (const [key, value] of Object.entries(item.grouped_values)) {
        merged[key] = (merged[key] ?? 0) + (value ?? 0);
      }
    }

    return Object.entries(merged)
      .map(([name, value]) => ({ name, value }))
      .sort((a, b) => b.value - a.value);
  }, [data]);

  return (
    <ChartContainer loading={loading}>
      <CirclePackingChart data={processedData} />
    </ChartContainer>
  );
};

export default CirclePackingChartSection;
