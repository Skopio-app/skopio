import TextSectionItem from "./TextSectionItem";

export type ActiveDay = { day: string | null; time: string | null };

interface MostActiveDaySectionProps {
  data: ActiveDay;
  loading: boolean;
}

const MostActiveDaySection = ({ data, loading }: MostActiveDaySectionProps) => {
  const resolvedData = data ?? { day: null, time: null };

  return (
    <TextSectionItem
      title="Most Active Day"
      text={
        resolvedData.day && resolvedData.time !== null
          ? `Your most active day was ${resolvedData.day} with ${resolvedData.time} of activity`
          : "No data found"
      }
      loading={loading}
    />
  );
};

export default MostActiveDaySection;
