import { useEffect, useState } from "react";
import ProjectBarChart from "../charts/ProjectBarChart";
import { useDashboardFilter } from "../stores/useDashboardFilter";
import WidgetCard from "../WidgetCard";
import { commands, SummaryQueryInput } from "../../../types/tauri.gen";
import { eachDayOfInterval, format } from "date-fns";

type BarChartData = {
  week: string;
  [project: string]: number | string;
};

const ProjectChartWidget = () => {
  const { startDate, endDate } = useDashboardFilter();
  const [data, setData] = useState<BarChartData[]>([]);
  const [keys, setKeys] = useState<string[]>([]);

  useEffect(() => {
    const run = async () => {
      const query: SummaryQueryInput = {
        start: startDate.toISOString(),
        end: endDate.toISOString(),
        include_afk: false,
      };
      console.log("[query]", query);

      const result = await commands.fetchProjectsSummary(query);

      console.log("[result]", result);

      if (!Array.isArray(result)) return;

      const days = eachDayOfInterval({ start: startDate, end: endDate });
      const dateLabels = days.map((d) => format(d, "MMM d"));

      const groupByProject: Record<string, number> = {};
      for (const item of result) {
        groupByProject[item.group_key] = item.total_seconds;
      }

      const chartData: BarChartData[] = [
        {
          week:
            dateLabels.length === 1
              ? dateLabels[0]
              : `${dateLabels[0]} → ${dateLabels.at(-1)}`,
          ...groupByProject,
        },
      ];

      setData(chartData);
      setKeys(Object.keys(groupByProject));
    };

    run();
  }, [startDate, endDate]);

  return (
    <WidgetCard title="Projects" onRemove={() => {}}>
      <ProjectBarChart data={data} keys={keys} />
    </WidgetCard>
  );
};

export default ProjectChartWidget;
