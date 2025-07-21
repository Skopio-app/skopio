import { useEffect, useMemo, useState } from "react";
import {
  BucketedSummaryInput,
  BucketTimeSummary,
  commands,
} from "../../../types/tauri.gen";
import { usePresetFilter } from "../stores/usePresetFilter";
import { formatDuration } from "../../../utils/time";
import { Skeleton } from "@skopio/ui";

interface EntityListProps {
  projectName: string;
}

const EntityList: React.FC<EntityListProps> = ({ projectName }) => {
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
      console.error("Error fetching entity list data: ", err);
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
    <div className="flex flex-col space-y-2">
      <h3 className="font-semibold">Entities</h3>
      {processedData.length === 0 && (
        <p className="w-full text-sm font-light">No entities found</p>
      )}
      {!loading && processedData ? (
        processedData.map((item) => {
          return (
            <div className="space-x-2 flex flex-row">
              <p className="text-xs font-light">{formatDuration(item.value)}</p>
              <p className="text-sm">{item.name}</p>
            </div>
          );
        })
      ) : (
        <Skeleton className="w-xl" />
      )}
    </div>
  );
};

export default EntityList;
