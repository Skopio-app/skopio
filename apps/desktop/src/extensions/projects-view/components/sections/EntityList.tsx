import { Group } from "../../../../types/tauri.gen";
import ItemsList from "../ItemsList";
import { useProjectSummaryData } from "../../hooks/useProjectSummaryData";

const EntityList = () => {
  const options = { group_by: "entity" as Group, mode: "list" as const };
  const { data, loading } = useProjectSummaryData(options);

  return <ItemsList title="Entities" data={data} loading={loading} />;
};

export default EntityList;
