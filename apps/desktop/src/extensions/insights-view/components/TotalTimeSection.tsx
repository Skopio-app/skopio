import TextSectionItem from "./TextSectionItem";
import { useYearFilter } from "../stores/useYearFilter";
import { commands, SummaryQueryInput } from "@/types/tauri.gen";
import { formatDuration } from "@/utils/time";

import { endOfYear, startOfYear } from "date-fns";
import { useQuery } from "@tanstack/react-query";

const TotalTimeSection = () => {
  const { year } = useYearFilter();

  const { data: time = null, isLoading } = useQuery({
    queryKey: ["totalTime", year],
    queryFn: async () => {
      const parsedYear = year
        ? parseInt(year.toString(), 10)
        : new Date().getFullYear();
      const start = startOfYear(new Date(parsedYear, 0, 1));
      const end = endOfYear(new Date(parsedYear, 0, 1));

      const input: SummaryQueryInput = {
        start: start.toISOString(),
        end: end.toISOString(),
      };
      return commands.fetchTotalTime(input);
    },
    select: (result): string => {
      return formatDuration(result);
    },
    enabled: Boolean(year),
  });

  return (
    <TextSectionItem
      title="Total time logged"
      text={
        time !== null ? `Total active time logged is ${time}` : "No data found"
      }
      loading={isLoading}
    />
  );
};

export default TotalTimeSection;
