import CalendarChart from "../../../components/CalendarChart";
import SectionContainer from "./SectionContainer";

const ActivitySection = () => {
  return (
    <SectionContainer title="Activity" loading={false}>
      <CalendarChart data={[]} />
    </SectionContainer>
  );
};

export default ActivitySection;
