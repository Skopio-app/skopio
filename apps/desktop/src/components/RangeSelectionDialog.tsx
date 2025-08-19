import * as Dialog from "@radix-ui/react-dialog";
import { Calendar, cn } from "@skopio/ui";
import { XIcon } from "lucide-react";
import { DATE_RANGE_LABELS, DateRangeType } from "../utils/time";
import { addDays, startOfDay } from "date-fns";
import { useState } from "react";

interface DateRangeDialogProps {
  selectedRange: DateRangeType;
  setSelectedRange: (range: DateRangeType) => void;
  pendingStart: Date;
  setPendingStart: (date: Date) => void;
  pendingEnd: Date;
  setPendingEnd: (date: Date) => void;
  setCustomStart: (date: Date) => void;
  setCustomEnd: (date: Date) => void;
  isCustom: boolean;
  searchParams: URLSearchParams;
  setSearchParams: (
    params: URLSearchParams,
    options?: { replace?: boolean },
  ) => void;
}

const RangeSelectionDialog: React.FC<DateRangeDialogProps> = ({
  selectedRange,
  setSelectedRange,
  pendingStart,
  setPendingStart,
  pendingEnd,
  setPendingEnd,
  setCustomStart,
  setCustomEnd,
  isCustom,
  searchParams,
  setSearchParams,
}) => {
  const [open, setOpen] = useState<boolean>(false);
  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
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
              Choose from a predefined range or pick a custom start and end
              date.
            </Dialog.Description>

            {DATE_RANGE_LABELS.map((range) => (
              <button
                key={range}
                className={cn(
                  "text-left w-full px-3 py-1.5 rounded-md hover:bg-muted text-gray-700",
                  selectedRange === range &&
                    "bg-muted font-semibold text-gray-900",
                )}
                onClick={() => {
                  setSelectedRange(range);
                  setOpen(false);
                }}
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
                    onSelect={(date) => date && setPendingEnd(date)}
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
                    setOpen(false);
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
  );
};

export default RangeSelectionDialog;
