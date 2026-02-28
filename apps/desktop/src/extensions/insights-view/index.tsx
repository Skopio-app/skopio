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
  BucketSummaryInput,
  commands,
  InsightQueryPayload,
  InsightResult,
  SummaryQueryInput,
  TimeBucket,
} from "@/types/tauri.gen";
import { useYearFilter } from "./stores/useYearFilter";
import TopLanguagesSection from "./components/TopLanguagesSection";
import MostActiveDaySection, {
  ActiveDay,
} from "./components/MostActiveDaySection";
import TotalTimeSection from "./components/TotalTimeSection";
import { useQueries, useQuery } from "@tanstack/react-query";
import type { AverageDaySectionData } from "./components/AverageDaySection";
import type { CalendarChartData } from "@/types/chart";
import { endOfYear, format, startOfYear } from "date-fns";
import { formatDuration } from "@/utils/time";
import { toast } from "sonner";
import { useEffect, useMemo } from "react";

const weekdayOrder = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
type TopN = [string, number][];

const InsightsView = () => {
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

  const [
    averageDayQuery,
    activityQuery,
    totalTimeQuery,
    topProjectsQuery,
    topLanguagesQuery,
    mostActiveDayQuery,
  ] = useQueries({
    queries: [
      {
        queryKey: ["aggregatedAverage", year],
        queryFn: async (): Promise<InsightResult> => {
          const query: InsightQueryPayload = {
            insightType: "aggregatedAverage",
            groupBy: "category",
            limit: 3,
            bucket: "day",
            insightRange: year,
          };
          return commands.fetchInsights(query);
        },
        select: (result: InsightResult): AverageDaySectionData => {
          if (!("aggregatedAverage" in result)) return { rows: [], keys: [] };
          const raw = result.aggregatedAverage;

          const sortedDays = Object.entries(raw).sort(([a], [b]) => {
            return weekdayOrder.indexOf(a) - weekdayOrder.indexOf(b);
          });

          const rows = sortedDays.map(([day, groups]) => {
            const row: AverageDaySectionData["rows"][number] = { label: day };
            for (const [group, value] of groups ?? []) {
              row[group] = value;
            }
            return row;
          });

          const keys = Array.from(
            new Set(
              Object.values(raw)
                .flatMap((pairs) => pairs ?? [])
                .map(([group]) => group),
            ),
          );

          return { rows, keys };
        },
        enabled: Boolean(year),
        placeholderData: undefined,
      },
      {
        queryKey: ["activityInsight", yearConfig.preset],
        queryFn: async (): Promise<CalendarChartData[]> => {
          const input: BucketSummaryInput = {
            preset: yearConfig.preset,
          };
          const result = await commands.fetchBucketedSummary(input);
          return result.map(({ bucket, groupedValues }) => ({
            day: bucket,
            value: groupedValues["Total"] ?? 0,
          }));
        },
        enabled: Boolean(year),
        placeholderData: undefined,
      },
      {
        queryKey: ["totalTime", year],
        queryFn: async (): Promise<number> => {
          const input: SummaryQueryInput = {
            start: yearConfig.start.toISOString(),
            end: yearConfig.end.toISOString(),
          };
          return commands.fetchTotalTime(input);
        },
        select: (result: number): string => formatDuration(result),
        enabled: Boolean(year),
        placeholderData: undefined,
      },
      {
        queryKey: ["topN", year, 3, "project"],
        queryFn: async (): Promise<InsightResult> => {
          const query: InsightQueryPayload = {
            insightType: "topN",
            insightRange: year,
            limit: 3,
            groupBy: "project",
          };

          return commands.fetchInsights(query);
        },
        select: (result: InsightResult): TopN =>
          "topN" in result ? result.topN : [],
        enabled: Boolean(year),
        placeholderData: undefined,
      },
      {
        queryKey: ["topN", year, 3, "language"],
        queryFn: async (): Promise<InsightResult> => {
          const query: InsightQueryPayload = {
            insightType: "topN",
            insightRange: year,
            limit: 3,
            groupBy: "language",
          };

          return commands.fetchInsights(query);
        },
        select: (result: InsightResult): TopN =>
          "topN" in result ? result.topN : [],
        enabled: Boolean(year),
        placeholderData: undefined,
      },
      {
        queryKey: ["mostActiveDay", year],
        queryFn: async (): Promise<InsightResult> => {
          const query: InsightQueryPayload = {
            insightType: "mostActiveDay",
            insightRange: year,
            bucket: "year",
          };
          return commands.fetchInsights(query);
        },
        select: (result: InsightResult): ActiveDay => {
          if (!("mostActiveDay" in result)) {
            return { day: null, time: null };
          }

          return {
            day: format(result.mostActiveDay.date, "EEE, MMM d"),
            time: formatDuration(result.mostActiveDay.total_duration),
          };
        },
        enabled: Boolean(year),
        placeholderData: undefined,
      },
    ],
  });

  useEffect(() => {
    for (const error of [
      activityQuery.error,
      topProjectsQuery.error,
      topLanguagesQuery.error,
    ]) {
      if (error) {
        toast.error((error as Error).message ?? String(error));
      }
    }
  }, [activityQuery.error, topProjectsQuery.error, topLanguagesQuery.error]);

  const topProjects = (topProjectsQuery.data ?? []).map(([project]) => project);
  const topLanguages = (topLanguagesQuery.data ?? []).map(
    ([language]) => language,
  );

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

      <AverageDaySection
        data={averageDayQuery.data ?? { rows: [], keys: [] }}
        loading={averageDayQuery.isLoading}
      />
      <ActivitySection
        data={activityQuery.data ?? []}
        loading={activityQuery.isLoading}
        start={yearConfig.start}
        end={yearConfig.end}
      />
      <TotalTimeSection
        time={totalTimeQuery.data ?? null}
        loading={totalTimeQuery.isLoading}
      />
      {/* <DailyAverageSection /> */}
      <TopProjectsSection
        projects={topProjects}
        loading={topProjectsQuery.isLoading}
      />
      <TopLanguagesSection
        languages={topLanguages}
        loading={topLanguagesQuery.isLoading}
      />
      <MostActiveDaySection
        data={mostActiveDayQuery.data ?? { day: null, time: null }}
        loading={mostActiveDayQuery.isLoading}
      />
    </div>
  );
};

export default InsightsView;
