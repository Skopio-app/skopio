import TextSectionItem from "./TextSectionItem";

interface TotalTimeSectionProps {
  time: string | null;
  loading: boolean;
}

const TotalTimeSection = ({ time, loading }: TotalTimeSectionProps) => {
  return (
    <TextSectionItem
      title="Total time logged"
      text={
        time !== null ? `Total active time logged is ${time}` : "No data found"
      }
      loading={loading}
    />
  );
};

export default TotalTimeSection;
