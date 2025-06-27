import { addDays, format, startOfDay } from "date-fns";
import { useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router-dom";
import {
  DATE_RANGE_LABELS,
  DateRangeType,
  formatDuration,
  getRangeDates,
  mapRangeToPreset,
} from "./helpers/dateRanges";
import "react-grid-layout/css/styles.css";
import "react-resizable/css/styles.css";
import { Layout, Responsive, WidthProvider } from "react-grid-layout";
import ProjectChartWidget from "./widgets/ProjectChartWidget";
import AppPieChartWidget from "./widgets/AppPieChartWidget";
import { useDashboardFilter } from "./stores/useDashboardFilter";
import LanguagePieChartWidget from "./widgets/LanguagePieChartWidget";
import CategoryChartWidget from "./widgets/CategoryChartWidget";
import EntityChartWidget from "./widgets/EntityChartWidget";
import RangeSelectionDialog from "./components/RangeSelectionDialog";
import ActivityChartWidget from "./widgets/ActivityChartWidget";
import { useTotalTime } from "./hooks/useTotalTime";
import { usePersistentLayout } from "./hooks/usePersistentLayout";

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
    const newPreset = mapRangeToPreset(selectedRange, startDate, endDate);
    useDashboardFilter.setState({ preset: newPreset });
  }, [selectedRange, startDate, endDate]);

  const formattedRange = useMemo(() => {
    const sameDay = format(startDate, "PPP") === format(endDate, "PPP");
    return sameDay
      ? format(startDate, "EEEE, MMMM do yyyy")
      : `${format(startDate, "EEE, MMM do yyyy")} - ${format(endDate, "EEE, MMM do yyyy")}`;
  }, [startDate, endDate]);

  const { total, loading } = useTotalTime(startDate, endDate);

  const formattedDuration = formatDuration(total);

  const defaultLayout: Layout[] = [
    { i: "projects", x: 0, y: 0, w: 6, h: 3 },
    { i: "apps", x: 6, y: 0, w: 6, h: 3 },
    { i: "languages", x: 0, y: 0, w: 6, h: 3 },
    { i: "categories", x: 6, y: 1, w: 6, h: 3 },
    { i: "entities", x: 0, y: 2, w: 6, h: 3 },
    { i: "activity", x: 6, y: 2, w: 6, h: 3 },
  ];

  const { layout, saveLayout, loaded } = usePersistentLayout(
    "default",
    defaultLayout,
  );
  const layouts = useMemo(() => ({ lg: layout }), [layout]);

  const layoutChildren = useMemo(
    () => [
      <div key="projects">
        <ProjectChartWidget />
      </div>,
      <div key="apps">
        <AppPieChartWidget />
      </div>,
      <div key="languages">
        <LanguagePieChartWidget />
      </div>,
      <div key="categories">
        <CategoryChartWidget />
      </div>,
      <div key="entities">
        <EntityChartWidget />
      </div>,
      <div key="activity">
        <ActivityChartWidget />
      </div>,
    ],
    [],
  );

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
        {loading ? (
          <span className="text-gray-400 animate-pulse">Loading...</span>
        ) : (
          <span className="text-gray-500">{formattedDuration}</span>
        )}
      </p>
      {loaded && (
        <ResponsiveGridLayout
          className="mt-4"
          layouts={layouts}
          breakpoints={{ lg: 1024, md: 768, sm: 480 }}
          cols={{ lg: 12, md: 10, sm: 6 }}
          rowHeight={100}
          isResizable
          isDraggable
          useCSSTransforms
          draggableHandle="#widget-drag-handle"
          onLayoutChange={(currLayout: Layout[]) => saveLayout(currLayout)}
        >
          {layoutChildren}
        </ResponsiveGridLayout>
      )}
    </main>
  );
};

export default DashboardView;
