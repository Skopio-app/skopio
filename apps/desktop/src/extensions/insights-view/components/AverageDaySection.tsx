import StackedBarChart from "../../../components/StackedBarChart";
import SectionContainer from "./SectionContainer";

const AverageDaySection = () => {
  return (
    <SectionContainer title="Weekday average" loading={false}>
      <StackedBarChart keys={[]} data={[]} />
    </SectionContainer>
  );
};

export default AverageDaySection;
