import { addDays, format, startOfDay } from "date-fns";
import { useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  DATE_RANGE_LABELS,
  DateRangeType,
  formatDuration,
  getRangeDates,
  mapRangeToPreset,
} from "./dateRanges";
import "react-grid-layout/css/styles.css";
import "react-resizable/css/styles.css";
import { Layouts, Responsive, WidthProvider } from "react-grid-layout";
import ProjectChartWidget from "./widgets/ProjectChartWidget";
import AppPieChartWidget from "./widgets/AppPieChartWidget";
import { useDashboardFilter } from "./stores/useDashboardFilter";
import LanguagePieChartWidget from "./widgets/LanguagePieChartWidget";
import CategoryChartWidget from "./widgets/CategoryChartWidget";
import EntityChartWidget from "./widgets/EntityChartWidget";
import RangeSelectionDialog from "./components/RangeSelectionDialog";
import { commands, SummaryQueryInput } from "../../types/tauri.gen";

const ResponsiveGridLayout = WidthProvider(
  Responsive,
) as React.ComponentType<any>;

const DashboardView = () => {
  const [customStart, setCustomStart] = useState<Date>(new Date());
  const [customEnd, setCustomEnd] = useState<Date>(new Date());

  const [pendingStart, setPendingStart] = useState<Date>(customStart);
  const [pendingEnd, setPendingEnd] = useState<Date>(customEnd);

  const [searchParams, setSearchParams] = useSearchParams();
  const paramRange = searchParams.get("range") as DateRangeType;
  const [selectedRange, setSelectedRange] = useState<DateRangeType>(
    paramRange && DATE_RANGE_LABELS.includes(paramRange)
      ? paramRange
      : DateRangeType.Today,
  );
  const [timeLogged, setTimeLogged] = useState<number>(0);

  const isCustom = selectedRange === DateRangeType.Custom;

  useEffect(() => {
    const params = new URLSearchParams(searchParams);
    params.set("range", selectedRange);
    setSearchParams(params, { replace: true });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedRange]);

  useEffect(() => {
    if (!isCustom) return;
    const maxEnd = addDays(startOfDay(customStart ?? 0), 30);
    if (customEnd) {
      if (customEnd > maxEnd) setCustomEnd(maxEnd);
    }
  }, [customStart, customEnd, isCustom]);

  const [startDate, endDate] = useMemo(
    () => getRangeDates(selectedRange, customStart, customEnd),
    [selectedRange, customStart, customEnd],
  );

  useEffect(() => {
    useDashboardFilter.setState({
      preset: mapRangeToPreset(selectedRange, customStart, customEnd),
    });
  }, [startDate, endDate]);

  const formattedRange = useMemo(() => {
    const sameDay = format(startDate, "PPP") === format(endDate, "PPP");
    return sameDay
      ? format(startDate, "PPP")
      : `${format(startDate, "PPP")} - ${format(endDate, "PPP")}`;
  }, [startDate, endDate]);

  useEffect(() => {
    const run = async () => {
      if (!startDate || !endDate) {
        console.warn("Missing start or end date!");
        return;
      }
      const input: SummaryQueryInput = {
        start: startDate.toISOString(),
        end: endDate.toISOString(),
        include_afk: false,
      };

      try {
        const time = await commands.fetchTotalTime(input);
        setTimeLogged(time);
      } catch (err) {
        console.error("Error fetching total time: ", err);
      }
    };

    run();
  }, [startDate, endDate]);

  const formattedDuration = formatDuration(timeLogged);

  const layouts: Layouts = {
    lg: [
      { i: "projects", x: 0, y: 0, w: 6, h: 3 },
      { i: "apps", x: 6, y: 0, w: 6, h: 3 },
      { i: "languages", x: 0, y: 0, w: 6, h: 3 },
      { i: "categories", x: 6, y: 1, w: 6, h: 3 },
      { i: "entities", x: 3, y: 2, w: 7, h: 3 },
    ],
  };

  return (
    <main className="p-6 space-y-4">
      <h1 className="text-2xl font-bold">
        <span className="text-gray-900">Activity for </span>
        <span className="text-gray-500">{formattedRange}</span>
      </h1>

      <p className="text-lg">
        <span className="text-gray-700 font-medium">
          Time logged for{" "}
          <RangeSelectionDialog
            selectedRange={selectedRange}
            setSelectedRange={setSelectedRange}
            pendingStart={pendingStart}
            setPendingStart={setPendingStart}
            pendingEnd={pendingEnd}
            setPendingEnd={setPendingEnd}
            setCustomStart={setCustomStart}
            setCustomEnd={setCustomStart}
            isCustom={isCustom}
            searchParams={searchParams}
            setSearchParams={setSearchParams}
          />
          :
        </span>{" "}
        <span className="text-gray-500">{formattedDuration}</span>
      </p>
      <ResponsiveGridLayout
        className="mt-4"
        layouts={layouts}
        breakpoints={{ lg: 1024, md: 768, sm: 480 }}
        cols={{ lg: 12, md: 10, sm: 6 }}
        rowHeight={100}
        isResizable
        isDraggable
        draggableHandle="#widget-drag-handle"
      >
        <div key="projects">
          <ProjectChartWidget />
        </div>
        <div key="apps">
          <AppPieChartWidget />
        </div>
        <div key="languages">
          <LanguagePieChartWidget />
        </div>
        <div key="categories">
          <CategoryChartWidget />
        </div>
        <div key="entities">
          <EntityChartWidget />
        </div>
      </ResponsiveGridLayout>
    </main>
  );
};

export default DashboardView;
