import { format } from "date-fns";
import { useEffect, useMemo } from "react";
import "react-grid-layout/css/styles.css";
import "react-resizable/css/styles.css";
import { Layout, Responsive, WidthProvider } from "react-grid-layout";
import ProjectChartWidget from "./widgets/ProjectChartWidget";
import AppPieChartWidget from "./widgets/AppPieChartWidget";
import { useDashboardFilter } from "./stores/useDashboardFilter";
import LanguagePieChartWidget from "./widgets/LanguagePieChartWidget";
import CategoryChartWidget from "./widgets/CategoryChartWidget";
import EntityChartWidget from "./widgets/EntityChartWidget";
import RangeSelectionDialog from "@/components/RangeSelectionDialog";
import ActivityChartWidget from "./widgets/ActivityChartWidget";
import { useTotalTime } from "./hooks/useTotalTime";
import { usePersistentLayout } from "./hooks/usePersistentLayout";
import { formatDuration } from "@/utils/time";
import { useDateRangeParams } from "@/hooks/useDateRangesParams";

const ResponsiveGridLayout = WidthProvider(
  Responsive,
) as React.ComponentType<any>;

const DashboardView = () => {
  const {
    selectedRange,
    setSelectedRange,
    startDate,
    endDate,
    isCustom,
    pendingStart,
    pendingEnd,
    setPendingStart,
    setPendingEnd,
    setCustomStart,
    setCustomEnd,
    applyPresetToStore,
  } = useDateRangeParams();

  useEffect(() => {
    applyPresetToStore((preset) => useDashboardFilter.setState({ preset }));
  }, [applyPresetToStore, selectedRange, startDate, endDate]);

  const sameDay = format(startDate, "PPP") === format(endDate, "PPP");
  const formattedRange = sameDay
    ? format(startDate, "EEEE, MMMM do yyyy")
    : `${format(startDate, "EEE, MMM do yyyy")} - ${format(endDate, "EEE, MMM do yyyy")}`;

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
    <main className="px-6 space-y-4">
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
            setCustomEnd={setCustomEnd}
            isCustom={isCustom}
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
