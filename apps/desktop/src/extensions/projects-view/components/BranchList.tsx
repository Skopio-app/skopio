import { useEffect, useMemo, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
} from "../../../types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";
import ItemsList from "./ItemsList";

interface BranchListProps {
  projectName: string;
}

const BranchList: React.FC<BranchListProps> = ({ projectName }) => {
  const [loading, setLoading] = useState(true);
  const [data, setData] = useState<BucketTimeSummary[]>([]);

  const { preset } = usePresetFilter();

  const query: BucketedSummaryInput = {
    preset,
    project_names: [projectName],
    group_by: "branch",
    include_afk: false,
  };

  const fetchData = async () => {
    try {
      const summary = await commands.fetchBucketedSummary(query);
      setData(summary);
    } catch (err) {
      console.error("Error fetching branch list data: ", err);
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

  return <ItemsList title="Branches" data={processedData} loading={loading} />;
};

export default BranchList;
