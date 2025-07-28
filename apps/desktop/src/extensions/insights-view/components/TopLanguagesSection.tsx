import { useTopNInsights } from "../hooks/useInsightsData";
import TextSectionItem from "./TextSectionItem";

const TopLanguagesSection = () => {
  const { data, loading } = useTopNInsights({ groupBy: "language" });

  const languages = data.map(([language]) => language);

  return (
    <TextSectionItem
      title="Top Languages"
      text={`Your top languages were ${languages.join(", ")}`}
      loading={loading}
    />
  );
};

export default TopLanguagesSection;
