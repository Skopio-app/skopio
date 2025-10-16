import { useYearFilter } from "../stores/useYearFilter";
import { commands, InsightQueryPayload } from "@/types/tauri.gen";
import SectionContainer from "./SectionContainer";
import StackedBarChart from "@/components/StackedBarChart";
import { useQuery } from "@tanstack/react-query";

type Row = { label: string; [key: string]: string | number };
type SelectResult = { rows: Row[]; keys: string[] };
const weekdayOrder = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

const AverageDaySection = () => {
  const { year } = useYearFilter();

  const {
    data = { rows: [], keys: [] },
    isFetching,
    isPending,
  } = useQuery({
    queryKey: ["aggregatedAverage", year],
    queryFn: async () => {
      const query: InsightQueryPayload = {
        insightType: "aggregatedAverage",
        groupBy: "category",
        limit: 3,
        bucket: "day",
        insightRange: year,
      };
      return commands.fetchInsights(query);
    },
    select: (result): SelectResult => {
      if (!("aggregatedAverage" in result)) return { rows: [], keys: [] };
      const raw = result.aggregatedAverage;

      const sortedDays = Object.entries(raw).sort(([a], [b]) => {
        return weekdayOrder.indexOf(a) - weekdayOrder.indexOf(b);
      });

      const rows: Row[] = sortedDays.map(([day, groups]) => {
        const row: Row = { label: day };
        for (const [group, value] of groups ?? []) {
          row[group] = value;
        }
        return row;
      });

      const keys = Array.from(
        new Set(
          Object.values(raw)
            .flatMap((pairs) => pairs ?? [])
            .map(([g]) => g),
        ),
      );

      return { rows, keys };
    },
    enabled: Boolean(year),
  });

  const loading = isFetching || isPending;

  return (
    <SectionContainer title="Weekday average" loading={loading}>
      <StackedBarChart keys={data.keys} data={data.rows} />
    </SectionContainer>
  );
};

export default AverageDaySection;
