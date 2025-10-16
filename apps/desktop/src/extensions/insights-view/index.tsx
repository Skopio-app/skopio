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
import {
  commands,
  InsightQueryPayload,
  InsightResult,
} from "@/types/tauri.gen";
import { useYearFilter } from "./stores/useYearFilter";
import TopLanguagesSection from "./components/TopLanguagesSection";
import MostActiveDaySection from "./components/MostActiveDaySection";
import TotalTimeSection from "./components/TotalTimeSection";
import { useQuery } from "@tanstack/react-query";

const InsightsView = () => {
  const { year } = useYearFilter();

  const { data: years = [], isLoading } = useQuery({
    queryKey: ["activeYears"],
    queryFn: (): Promise<InsightResult> => {
      const query: InsightQueryPayload = {
        insightType: "activeYears",
      };
      return commands.fetchInsights(query);
    },
    select: (result): number[] =>
      "activeYears" in result ? result.activeYears : [],
  });

  return (
    <div className="flex flex-col mx-3 mb-4 space-y-5">
      <h2 className="text-foreground font-semibold mb-4 mt-3 text-lg">
        Insights
      </h2>
      <Select
        value={year}
        onValueChange={(year) => {
          useYearFilter.setState({ year });
        }}
        disabled={isLoading}
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
