import TextSectionItem from "./TextSectionItem";
import { commands, InsightQueryPayload } from "@/types/tauri.gen";
import { useYearFilter } from "../stores/useYearFilter";
import { format } from "date-fns";
import { formatDuration } from "@/utils/time";
import { useQuery } from "@tanstack/react-query";

const MostActiveDaySection = () => {
  const { year } = useYearFilter();

  type ActiveDay = { day: string | null; time: string | null };

  const {
    data = { day: null, time: null },
    isFetching,
    isPending,
  } = useQuery({
    queryKey: ["mostActiveDay", year],
    queryFn: async () => {
      const query: InsightQueryPayload = {
        insightType: "mostActiveDay",
        insightRange: year,
        bucket: "year",
      };
      return commands.fetchInsights(query);
    },
    select: (result): ActiveDay => {
      if (!("mostActiveDay" in result)) {
        return { day: null, time: null };
      }
      const formattedDate = format(result.mostActiveDay.date, "EEE, MMM d");
      const formattedDuration = formatDuration(
        result.mostActiveDay.total_duration,
      );

      return { day: formattedDate, time: formattedDuration };
    },
    enabled: Boolean(year),
  });

  const loading = isFetching || isPending;

  return (
    <TextSectionItem
      title="Most Active Day"
      text={
        data.day && data.time !== null
          ? `Your most active day was ${data.day} with ${data.time} of activity`
          : "No data found"
      }
      loading={loading}
    />
  );
};

export default MostActiveDaySection;
