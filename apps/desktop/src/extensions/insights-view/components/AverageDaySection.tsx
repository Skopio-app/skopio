import { useEffect, useState } from "react";
import { useYearFilter } from "../stores/useYearFilter";
import { commands, InsightQueryPayload } from "../../../types/tauri.gen";
import SectionContainer from "./SectionContainer";
import StackedBarChart from "../../../components/StackedBarChart";
import { toast } from "sonner";

const AverageDaySection = () => {
  const [results, setResults] = useState<
    { label: string; [key: string]: string | number }[]
  >([]);
  const [keys, setKeys] = useState<string[]>([]);
  const { year } = useYearFilter();
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    const query: InsightQueryPayload = {
      insightType: "aggregatedAverage",
      groupBy: "category",
      limit: 3,
      bucket: "day",
      insightRange: year,
    };

    if (year.length === 0) return;
    commands
      .fetchInsights(query)
      .then((result) => {
        if ("aggregatedAverage" in result) {
          const raw = result.aggregatedAverage;

          const days = Object.entries(raw);

          const weekdayOrder = [
            "Mon",
            "Tue",
            "Wed",
            "Thu",
            "Fri",
            "Sat",
            "Sun",
          ];

          const sortedDays = [...Object.entries(raw)].sort(([a], [b]) => {
            return weekdayOrder.indexOf(a) - weekdayOrder.indexOf(b);
          });

          const chartData: { label: string; [key: string]: string | number }[] =
            sortedDays.map(([day, groups]) => {
              const row: { label: string; [key: string]: string | number } = {
                label: day,
              };
              for (const [group, value] of groups ?? []) {
                row[group] = value;
              }
              return row;
            });

          const allKeys = new Set<string>();
          for (const [, groups] of days) {
            for (const [group] of groups ?? []) {
              allKeys.add(group);
            }
          }

          setResults(chartData);
          setKeys(Array.from(allKeys));
        }
      })
      .catch(toast.error)
      .finally(() => setLoading(false));
  }, [year]);

  return (
    <SectionContainer title="Weekday average" loading={loading}>
      <StackedBarChart keys={keys} data={results} />
    </SectionContainer>
  );
};

export default AverageDaySection;
