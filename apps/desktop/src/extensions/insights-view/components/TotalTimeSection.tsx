import { useEffect, useState } from "react";
import TextSectionItem from "./TextSectionItem";
import { useYearFilter } from "../stores/useYearFilter";
import { commands, SummaryQueryInput } from "@/types/tauri.gen";
import { formatDuration } from "@/utils/time";
import { toast } from "sonner";
import { endOfYear, startOfYear } from "date-fns";

const TotalTimeSection = () => {
  const [loading, setLoading] = useState(false);
  const { year } = useYearFilter();
  const [time, setTime] = useState<string | null>(null);

  useEffect(() => {
    const fetchTotalTime = async () => {
      const parsedYear = year
        ? parseInt(year.toString(), 10)
        : new Date().getFullYear();
      const start = startOfYear(new Date(parsedYear, 0, 1));
      const end = endOfYear(new Date(parsedYear, 0, 1));

      const input: SummaryQueryInput = {
        start: start.toISOString(),
        end: end.toISOString(),
      };

      commands
        .fetchTotalTime(input)
        .then((time) => {
          const formattedTime = formatDuration(time);
          setTime(formattedTime);
        })
        .catch(toast.error)
        .finally(() => setLoading(false));
    };

    fetchTotalTime();
  }, [year]);

  return (
    <TextSectionItem
      title="Total time logged"
      text={
        time !== null ? `Total active time logged is ${time}` : "No data found"
      }
      loading={loading}
    />
  );
};

export default TotalTimeSection;
