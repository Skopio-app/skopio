import { useTopNInsights } from "../hooks/useInsightsData";
import TextSectionItem from "./TextSectionItem";

const TopProjectsSection = () => {
  const { data, loading } = useTopNInsights({ groupBy: "project" });

  const projects = data.map(([project]) => project);

  return (
    <TextSectionItem
      title="Top projects"
      text={`Your top projects were ${projects.join(", ")}`}
      loading={loading}
    />
  );
};

export default TopProjectsSection;
