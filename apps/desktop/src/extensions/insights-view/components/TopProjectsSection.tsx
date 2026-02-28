import TextSectionItem from "./TextSectionItem";

interface TopProjectsSectionProps {
  projects: string[];
  loading: boolean;
}

const TopProjectsSection = ({ projects, loading }: TopProjectsSectionProps) => {
  return (
    <TextSectionItem
      title="Top projects"
      text={
        projects.length === 0
          ? "No projects found"
          : `Your top projects were ${projects.join(", ")}`
      }
      loading={loading}
    />
  );
};

export default TopProjectsSection;
