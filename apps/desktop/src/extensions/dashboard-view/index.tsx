import * as Dialog from "@radix-ui/react-dialog";
import { Calendar, cn } from "@skopio/ui";
import { format } from "date-fns";
import { useState } from "react";
import { XIcon } from "lucide-react";

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

const predefinedRanges = [
  "Today",
  "Yesterday",
  "Last 7 days",
  "Last 14 days",
  "Last 30 days",
  "This week",
  "Last week",
  "This month",
  "Last month",
  "Custom range",
];

const DashboardView = () => {
  const now = Date.now();
  const formattedDate = format(now, "EEEE, MMMM do, yyyy");
  const timeLogged = 4567;
  const formattedDuration = formatDuration(timeLogged);

  const [selectedRange, setSelectedRange] = useState("Today");
  const [customStart, setCustomStart] = useState<Date | undefined>(new Date());
  const [customEnd, setCustomEnd] = useState<Date | undefined>(new Date());

  const isCustom = selectedRange === "Custom range";

  return (
    <main className="p-6 space-y-4">
      <h1 className="text-2xl font-bold">
        <span className="text-gray-900">Activity for </span>
        <span className="text-gray-500">{formattedDate}</span>
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

                  {predefinedRanges.map((range) => (
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
                  <div className="flex gap-6">
                    {" "}
                    <div className="space-y-2">
                      <label className="block text-sm font-medium text-gray-700 mt-4">
                        Start Date
                      </label>
                      <Calendar
                        mode="single"
                        className="rounded-md border shadow-sm p-6"
                        selected={customStart}
                        onSelect={setCustomStart}
                        captionLayout="dropdown"
                      />
                    </div>
                    <div className="space-y-2">
                      <label className="block text-sm font-medium text-gray-700 mb-1 mt-4">
                        End Date
                      </label>
                      <Calendar
                        mode="single"
                        className="rounded-md border shadow-sm p-6"
                        selected={customEnd}
                        onSelect={setCustomEnd}
                        captionLayout="dropdown"
                      />
                    </div>
                  </div>
                )}

                {/* Close Button */}
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
    </main>
  );
};

export default DashboardView;
