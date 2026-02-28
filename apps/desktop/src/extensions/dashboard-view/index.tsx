import { format } from "date-fns";
import { Ref, useEffect, useMemo } from "react";
import "react-grid-layout/css/styles.css";
import "react-resizable/css/styles.css";
import {
  Layout,
  Responsive,
  ResizeHandleAxis,
  useContainerWidth,
} from "react-grid-layout";
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
import { ChevronRight } from "lucide-react";
import { DashboardLayouts } from "@/db/skopioDB";

const DashboardView = () => {
  const { width, containerRef, mounted } = useContainerWidth();
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

  const defaultLayouts = useMemo<DashboardLayouts>(
    () => ({
      lg: [
        { i: "projects", x: 0, y: 0, w: 6, h: 3 },
        { i: "apps", x: 6, y: 0, w: 6, h: 3 },
        { i: "languages", x: 0, y: 0, w: 6, h: 3 },
        { i: "categories", x: 6, y: 1, w: 6, h: 3 },
        { i: "entities", x: 0, y: 2, w: 6, h: 3 },
        { i: "activity", x: 6, y: 2, w: 6, h: 3 },
      ],
    }),
    [],
  );

  const { layouts, saveLayouts, loaded } = usePersistentLayout(
    "default",
    defaultLayouts,
  );

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
        <span className="text-foreground">Activity for </span>
        <span className="text-muted-foreground">{formattedRange}</span>
      </h1>

      <p className="text-lg">
        <span className="text-accent-foreground font-medium">
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
          <span className="text-muted-foreground animate-pulse">
            Loading...
          </span>
        ) : (
          <span className="text-muted-foreground">{formattedDuration}</span>
        )}
      </p>
      <div ref={containerRef}>
        {loaded && mounted && (
          <Responsive
            className="mt-4 scroll-hidden"
            width={width}
            layouts={layouts}
            breakpoints={{ lg: 1024, md: 768, sm: 480 }}
            cols={{ lg: 12, md: 10, sm: 6 }}
            rowHeight={100}
            dragConfig={{ enabled: true, handle: ".widget-drag-handle" }}
            resizeConfig={{
              enabled: true,
              handleComponent: (
                _handleAxis: ResizeHandleAxis,
                ref: Ref<HTMLElement>,
              ) => (
                <div
                  ref={ref as Ref<HTMLDivElement>}
                  className="absolute bottom-1 right-1 w-3 h-3 text-muted-foreground rounded cursor-se-resize"
                >
                  <ChevronRight
                    className="rotate-45 size-3"
                    strokeWidth={3.5}
                  />
                </div>
              ),
            }}
            onLayoutChange={(_currentLayout: Layout, allLayouts) =>
              saveLayouts(allLayouts)
            }
          >
            {layoutChildren}
          </Responsive>
        )}
      </div>
    </main>
  );
};

export default DashboardView;
