import { useEffect, useState } from "react";
import CalendarChart from "../../../components/CalendarChart";
import SectionContainer from "./SectionContainer";
import { CalendarChartData } from "../../../types/types";
import {
  BucketedSummaryInput,
  commands,
  TimeRangePreset,
} from "../../../types/tauri.gen";
import { endOfYear, startOfYear } from "date-fns";
import { useYearFilter } from "../stores/useYearFilter";

const ActivitySection = () => {
  const [data, setData] = useState<CalendarChartData[]>([]);
  const { year } = useYearFilter();

  const getYearPreset = (
    year?: string | number,
  ): {
    start: Date;
    end: Date;
    preset: TimeRangePreset;
  } => {
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
          bucket: "day",
        },
      },
    };
  };

  const yearConfig = getYearPreset(year);

  useEffect(() => {
    const fetchActivityData = async () => {
      try {
        const input: BucketedSummaryInput = {
          preset: yearConfig.preset,
          include_afk: false,
        };

        const result = await commands.fetchBucketedSummary(input);
        if (!Array.isArray(result)) return;

        const values: CalendarChartData[] = result.map(
          ({ bucket, grouped_values }) => ({
            day: bucket,
            value: grouped_values["Total"] ?? 0,
          }),
        );
        setData(values);
      } catch (err) {
        console.error("The error: ", err);
      }
    };

    fetchActivityData();
  }, [year]);

  return (
    <SectionContainer title="Activity" loading={false}>
      <CalendarChart
        data={data}
        start={yearConfig.start}
        end={yearConfig.end}
      />
    </SectionContainer>
  );
};

export default ActivitySection;
