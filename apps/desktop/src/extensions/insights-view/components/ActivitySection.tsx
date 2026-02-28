import CalendarChart from "@/components/CalendarChart";
import SectionContainer from "./SectionContainer";
import { CalendarChartData } from "@/types/chart";

interface ActivitySectionProps {
  data: CalendarChartData[];
  loading: boolean;
  start: Date;
  end: Date;
}

const ActivitySection = ({
  data,
  loading,
  start,
  end,
}: ActivitySectionProps) => {
  return (
    <SectionContainer
      title="Activity"
      loading={loading}
      skeletonVariant="calendar"
    >
      <CalendarChart data={data} start={start} end={end} />
    </SectionContainer>
  );
};

export default ActivitySection;
