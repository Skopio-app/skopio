import { useEffect, useState } from "react";
import TextSectionItem from "./TextSectionItem";
import { useYearFilter } from "../stores/useYearFilter";
import { commands, InsightQueryPayload } from "@/types/tauri.gen";
import { toast } from "sonner";

const DailyAverageSection = () => {
  const [loading, setLoading] = useState(true);
  const { year } = useYearFilter();

  useEffect(() => {
    const fetch = () => {
      const query: InsightQueryPayload = {
        insightType: "aggregatedAverage",
        insightRange: year,
        bucket: "day",
      };
      commands
        .fetchInsights(query)
        .then((result) => {
          if ("aggregatedAverage" in result) {
            console.log(result.aggregatedAverage);
          }
        })
        .catch(toast.error)
        .finally(() => setLoading(false));
    };

    fetch();
  }, [year]);

  return (
    <TextSectionItem
      title="Daily average"
      text="2hr 15min per day"
      loading={loading}
    />
  );
};

export default DailyAverageSection;
