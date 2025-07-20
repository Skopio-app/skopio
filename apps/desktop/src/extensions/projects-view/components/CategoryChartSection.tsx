import { useEffect, useState } from "react";
import { BucketedSummaryInput, commands } from "../../../types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";
import ChartContainer from "./ChartContainer";
import StackedBarChart from "../../../components/StackedBarChart";
import { BarChartData } from "../../../types/types";

interface CategoryChartSectionProps {
  projectName: string;
}

// TODO: Refactor and centralize bucket parsing logic.
const CategoryChartSection: React.FC<CategoryChartSectionProps> = ({
  projectName,
}) => {
  const [loading, setLoading] = useState(true);
  const [data, setData] = useState<BarChartData[]>([]);
  const [keys, setKeys] = useState<string[]>([]);

  const { preset } = usePresetFilter();

  const fetchData = async () => {
    const query: BucketedSummaryInput = {
      preset,
      project_names: [projectName],
      group_by: "category",
      include_afk: false,
    };

    try {
      const summary = await commands.fetchBucketedSummary(query);
      console.log("The summary: ", summary);
      const grouped: {
        date: Date;
        label: string;
        values: Record<string, number>;
      }[] = [];
      const allKeys = new Set<string>();
      const totalPerKey: Record<string, number> = {};

      for (const { bucket, grouped_values } of summary) {
        const date = new Date(bucket);
        const values: Record<string, number> = {};

        for (const [category, seconds] of Object.entries(grouped_values)) {
          const value = seconds ?? 0;
          values[category] = seconds ?? 0;
          totalPerKey[category] = (totalPerKey[category] || 0) + value;
          allKeys.add(category);
        }

        grouped.push({ date, label: bucket, values });
      }

      grouped.sort((a, b) => a.date.getTime() - b.date.getTime());

      const sortedKeys = Array.from(allKeys).sort(
        (a, b) => (totalPerKey[b] ?? 0) - (totalPerKey[a] ?? 0),
      );
      setKeys(sortedKeys);
      const chartData: BarChartData[] = grouped.map(({ label, values }) => ({
        label,
        ...values,
      }));
      setData(chartData);
    } catch (err) {
      console.error("Error fetching category chart summary data: ", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, [projectName, preset]);

  return (
    <ChartContainer loading={loading}>
      <StackedBarChart data={data} keys={keys} />
    </ChartContainer>
  );
};

export default CategoryChartSection;
