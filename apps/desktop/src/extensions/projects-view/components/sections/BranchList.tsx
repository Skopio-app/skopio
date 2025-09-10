import { Group } from "@/types/tauri.gen";
import ItemsList from "../ItemsList";
import { useProjectSummaryData } from "../../hooks/useProjectSummaryData";

const BranchList = () => {
  const options = { group_by: "branch" as Group, mode: "list" as const };
  const { data, loading } = useProjectSummaryData(options);

  return <ItemsList title="Branches" data={data} loading={loading} />;
};

export default BranchList;
