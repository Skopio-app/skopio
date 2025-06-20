import * as Dialog from "@radix-ui/react-dialog";
import { Calendar, cn } from "@skopio/ui";
import { addDays, format, startOfDay } from "date-fns";
import { useEffect, useMemo, useState } from "react";
import { XIcon } from "lucide-react";
import { useSearchParams } from "react-router-dom";
import { DATE_RANGE_LABELS, DateRangeType, getRangeDates } from "./dateRanges";
import "react-grid-layout/css/styles.css";
import "react-resizable/css/styles.css";
import { Layouts, Responsive, WidthProvider } from "react-grid-layout";
import ProjectChartWidget from "./widgets/ProjectChartWidget";
import AppPieChartWidget from "./widgets/AppPieChartWidget";
import { useDashboardFilter } from "./stores/useDashboardFilter";

const ResponsiveGridLayout = WidthProvider(
  Responsive,
) as React.ComponentType<any>;

const DashboardView = () => {
  const [customStart, setCustomStart] = useState<Date>(new Date());
  const [customEnd, setCustomEnd] = useState<Date>(new Date());

  const [pendingStart, setPendingStart] = useState<Date>(customStart);
  const [pendingEnd, setPendindEnd] = useState<Date>(customEnd);

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
    useDashboardFilter.setState({
      startDate,
      endDate,
    });
  }, [startDate, endDate]);

  const formattedRange = useMemo(() => {
    const sameDay = format(startDate, "PPP") === format(endDate, "PPP");
    return sameDay
      ? format(startDate, "PPP")
      : `${format(startDate, "PPP")} - ${format(endDate, "PPP")}`;
  }, [startDate, endDate]);

  const formatDuration = (seconds: number): string => {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;

    const padded = (n: number) => String(n).padStart(2, "0");
    const hrStr = `${hrs}h`;
    const minStr = `${padded(mins)}m`;
    const secStr = `${padded(secs)}s`;
    if (hrs > 0) {
      return `${hrStr} ${minStr} ${secStr}`;
    } else if (mins > 0) {
      return `${mins} ${secStr}`;
    } else {
      return `${secStr}`;
    }
  };

  const timeLogged = 4567;
  const formattedDuration = formatDuration(timeLogged);

  const layouts: Layouts = {
    lg: [
      { i: "weekly-project", x: 0, y: 0, w: 6, h: 4 },
      { i: "app-duration", x: 1, y: 0, w: 4, h: 6 },
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
          <Dialog.Root>
            <Dialog.Trigger asChild>
              <button className="underline decoration-dotted underline-offset-4 text-blue-600 hover:text-blue-800">
                {selectedRange.toLowerCase()}
              </button>
            </Dialog.Trigger>

            <Dialog.Portal>
              <Dialog.Overlay className="fixed inset-0 bg-black/50 z-50" />
              <Dialog.Content
                className={cn(
                  "fixed top-1/2 left-1/2 z-50 w-full translate-x-[-50%] translate-y-[-50%] rounded-lg border border-border bg-white p-6 shadow-lg",
                  "grid grid-cols-[auto_1fr] gap-x-8 items-start",
                  isCustom ? "max-w-4xl" : "max-w-xl",
                )}
              >
                <div className="flex flex-col gap-2 min-w-[180px] flex-shrink-0">
                  <Dialog.Title className="text-lg font-semibold text-gray-800 mb-2">
                    Select a date range
                  </Dialog.Title>
                  <Dialog.Description className="text-sm text-muted-foreground sr-only">
                    Choose from a predefined range or pick a custom start and
                    end date.
                  </Dialog.Description>

                  {DATE_RANGE_LABELS.map((range) => (
                    <button
                      key={range}
                      className={cn(
                        "text-left w-full px-3 py-1.5 rounded-md hover:bg-muted text-gray-700",
                        selectedRange === range &&
                          "bg-muted font-semibold text-gray-900",
                      )}
                      onClick={() => setSelectedRange(range)}
                    >
                      {range}
                    </button>
                  ))}
                </div>

                {isCustom && (
                  <div className="flex flex-col gap-4">
                    {" "}
                    <div className="flex gap-6">
                      <div className="space-y-2">
                        <label className="block text-sm font-medium text-gray-700 mb-2 mt-4">
                          Start Date
                        </label>
                        <Calendar
                          mode="single"
                          className="rounded-md border shadow-sm p-6"
                          selected={pendingStart}
                          onSelect={(date) => date && setPendingStart(date)}
                          captionLayout="dropdown"
                        />
                      </div>
                      <div className="space-y-2">
                        <label className="block text-sm font-medium text-gray-700 mb-2 mt-4">
                          End Date
                        </label>
                        <Calendar
                          mode="single"
                          className="rounded-md border shadow-sm p-6"
                          selected={pendingEnd}
                          onSelect={(date) => date && setPendindEnd(date)}
                          captionLayout="dropdown"
                          hidden={{
                            before: pendingStart,
                            after: addDays(startOfDay(pendingStart), 30),
                          }}
                        />
                      </div>
                    </div>
                    <div>
                      <button
                        disabled={pendingStart > pendingEnd}
                        onClick={() => {
                          setCustomStart(pendingStart);
                          setCustomEnd(pendingEnd);
                          const params = new URLSearchParams(searchParams);
                          params.set("range", DateRangeType.Custom);
                          setSearchParams(params, { replace: true });
                        }}
                        className={cn(
                          "mt-2 inline-block px-4 py-2 rounded-md font-medium transition-all text-white",
                          pendingStart > pendingEnd
                            ? "bg-red-400 cursor-not-allowed"
                            : "bg-blue-600 hover:bg-blue-700",
                        )}
                      >
                        Apply
                      </button>
                      {pendingStart > pendingEnd && (
                        <p className="text-sm text-red-500 mt-1">
                          Start date must be before and equal to end date.
                        </p>
                      )}
                    </div>
                  </div>
                )}

                <Dialog.Close asChild>
                  <button className="absolute top-4 right-4 text-gray-400 hover:text-gray-600 rounded-full p-1 transition-colors duration-200 focus:outline-none focus:ring-2 focus:ring-blue-500">
                    <XIcon className="w-5 h-5" />
                    <span className="sr-only">Close</span>
                  </button>
                </Dialog.Close>
              </Dialog.Content>
            </Dialog.Portal>
          </Dialog.Root>
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
        <div key="weekly-project">
          <ProjectChartWidget />
        </div>
        <div key="app-duration">
          <AppPieChartWidget />
        </div>
      </ResponsiveGridLayout>
    </main>
  );
};

export default DashboardView;
