import { useMemo } from "react";
import CalendarChart from "@/components/CalendarChart";
import SectionContainer from "./SectionContainer";
import { CalendarChartData } from "@/types/chart";
import { BucketSummaryInput, commands, TimeBucket } from "@/types/tauri.gen";
import { endOfYear, startOfYear } from "date-fns";
import { useYearFilter } from "../stores/useYearFilter";
import { toast } from "sonner";
import { useQuery } from "@tanstack/react-query";

const ActivitySection = () => {
  const { year } = useYearFilter();

  const yearConfig = useMemo(() => {
    const parsedYear = year
      ? parseInt(year.toString(), 10)
      : new Date().getFullYear();
    const start = startOfYear(new Date(parsedYear, 0, 1));
    const end = endOfYear(new Date(parsedYear, 0, 1));

    return {
      start,
      end,
      preset: {
        custom: {
          start: start.toISOString(),
          end: end.toISOString(),
          bucket: "day" as TimeBucket,
        },
      },
    };
  }, [year]);

  const {
    data: calendarData = [],
    isLoading,
    error,
  } = useQuery<CalendarChartData[]>({
    queryKey: ["activityInsight", yearConfig.preset],
    queryFn: async () => {
      const input: BucketSummaryInput = {
        preset: yearConfig.preset,
      };
      const result = await commands.fetchBucketedSummary(input);
      return result.map(({ bucket, groupedValues }) => ({
        day: bucket,
        value: groupedValues["Total"] ?? 0,
      }));
    },
    enabled: Boolean(yearConfig.preset),
  });

  if (error) {
    toast.error(
      `Error fetching activity section data: ${(error as Error).message}`,
    );
  }

  return (
    <SectionContainer title="Activity" loading={isLoading}>
      <CalendarChart
        data={calendarData}
        start={yearConfig.start}
        end={yearConfig.end}
      />
    </SectionContainer>
  );
};

export default ActivitySection;
