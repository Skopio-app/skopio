import SectionContainer from "./SectionContainer";
import StackedBarChart from "@/components/StackedBarChart";

type Row = { label: string; [key: string]: string | number };
export type AverageDaySectionData = { rows: Row[]; keys: string[] };

interface AverageDaySectionProps {
  data: AverageDaySectionData;
  loading: boolean;
}

const AverageDaySection = ({ data, loading }: AverageDaySectionProps) => {
  const resolvedData = data ?? { rows: [], keys: [] };

  return (
    <SectionContainer title="Weekday average" loading={loading}>
      <StackedBarChart keys={resolvedData.keys} data={resolvedData.rows} />
    </SectionContainer>
  );
};

export default AverageDaySection;
