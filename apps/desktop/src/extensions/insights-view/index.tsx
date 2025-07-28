import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@skopio/ui";
import AverageDaySection from "./components/AverageDaySection";
import ActivitySection from "./components/ActivitySection";
import TopProjectsSection from "./components/TopProjectsSection";
import { useEffect, useState } from "react";
import { commands, InsightQueryPayload } from "../../types/tauri.gen";
import { useYearFilter } from "./stores/useYearFilter";
import TopLanguagesSection from "./components/TopLanguagesSection";
import MostActiveDaySection from "./components/MostActiveDaySection";
import TotalTimeSection from "./components/TotalTimeSection";

const InsightsView = () => {
  const [years, setYears] = useState<number[]>([]);
  const [selectedYear, setSelectedYear] = useState<string | null>(null);

  useEffect(() => {
    const query: InsightQueryPayload = {
      insightType: "activeYears",
    };

    commands
      .fetchInsights(query)
      .then((result) => {
        if ("activeYears" in result) {
          setYears(result.activeYears);
          if (result.activeYears.length > 0) {
            useYearFilter.setState({ year: String(result.activeYears[0]) });
            setSelectedYear(String(result.activeYears[0]));
          }
        }
      })
      .catch(console.error);
  }, []);

  return (
    <div className="flex flex-col m-3 space-y-5">
      <h2 className="text-neutral-800 font-semibold mb-4 mt-3 text-lg">
        Insights
      </h2>
      <Select
        value={String(selectedYear)}
        onValueChange={(year) => {
          setSelectedYear(year);
          useYearFilter.setState({ year });
        }}
      >
        <SelectTrigger className="w-[180px]">
          <SelectValue placeholder="Select a year" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectLabel>Year</SelectLabel>
            {years.map((year) => (
              <SelectItem key={year} value={String(year)}>
                {year}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>

      <AverageDaySection />
      <ActivitySection />
      <TotalTimeSection />
      {/* <DailyAverageSection /> */}
      <TopProjectsSection />
      <TopLanguagesSection />
      <MostActiveDaySection />
    </div>
  );
};

export default InsightsView;
