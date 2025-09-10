import { useEffect, useState } from "react";
import TextSectionItem from "./TextSectionItem";
import { commands, InsightQueryPayload } from "@/types/tauri.gen";
import { useYearFilter } from "../stores/useYearFilter";
import { toast } from "sonner";
import { format } from "date-fns";
import { formatDuration } from "@/utils/time";

const MostActiveDaySection = () => {
  const [loading, setLoading] = useState(true);
  const [day, setDay] = useState<string | null>(null);
  const [time, setTime] = useState<string | null>(null);
  const { year } = useYearFilter();

  useEffect(() => {
    const fetch = async () => {
      const query: InsightQueryPayload = {
        insightType: "mostActiveDay",
        insightRange: year,
        bucket: "year",
      };

      try {
        if (year.length === 0) return;
        const result = await commands.fetchInsights(query);
        if ("mostActiveDay" in result) {
          const formattedDate = format(result.mostActiveDay.date, "EEE, MMM d");
          const formattedDuration = formatDuration(
            result.mostActiveDay.total_duration,
          );

          setDay(formattedDate);
          setTime(formattedDuration);
        }
      } catch (err) {
        toast.error(`Failed to fetch most active day: ${err}`);
      } finally {
        setLoading(false);
      }
    };

    fetch();
  }, [year]);

  return (
    <TextSectionItem
      title="Most Active Day"
      text={
        day && time !== null
          ? `Your most active day was ${day} with ${time} of activity`
          : "No data found"
      }
      loading={loading}
    />
  );
};

export default MostActiveDaySection;
