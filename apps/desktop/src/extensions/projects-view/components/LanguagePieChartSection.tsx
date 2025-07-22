import { useEffect, useMemo, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
} from "../../../types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";
import { PieChartData } from "../../../types/types";
import ChartContainer from "./ChartContainer";
import CustomPieChart from "../../../components/CustomPieChart";

interface LanguagePieChartSectionProps {
  projectName: string;
}

const LanguagePieChartSection: React.FC<LanguagePieChartSectionProps> = ({
  projectName,
}) => {
  const [loading, setLoading] = useState(true);
  const [data, setData] = useState<BucketTimeSummary[]>([]);

  const { preset } = usePresetFilter();

  const query: BucketedSummaryInput = {
    preset,
    project_names: [projectName],
    group_by: "language",
    include_afk: false,
  };

  const fetchData = async () => {
    try {
      const summary = await commands.fetchBucketedSummary(query);
      setData(summary);
    } catch (err) {
      console.error("Error fetching language pie chart summary data: ", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, [projectName, preset]);

  const processedData = useMemo<PieChartData[]>(() => {
    const merged: Record<string, number> = {};

    for (const item of data) {
      for (const [key, value] of Object.entries(item.grouped_values)) {
        merged[key] = (merged[key] ?? 0) + (value ?? 0);
      }
    }

    return Object.entries(merged).map(([id, value]) => ({
      id,
      label: id,
      value,
    }));
  }, [data]);

  return (
    <ChartContainer title="Languages" loading={loading}>
      <CustomPieChart data={processedData} />
    </ChartContainer>
  );
};

export default LanguagePieChartSection;
