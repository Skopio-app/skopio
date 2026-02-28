import TextSectionItem from "./TextSectionItem";

interface TopLanguagesSectionProps {
  languages: string[];
  loading: boolean;
}

const TopLanguagesSection = ({
  languages,
  loading,
}: TopLanguagesSectionProps) => {
  return (
    <TextSectionItem
      title="Top Languages"
      text={
        languages.length === 0
          ? "No languages found"
          : `Your top languages were ${languages.join(", ")}`
      }
      loading={loading}
    />
  );
};

export default TopLanguagesSection;
