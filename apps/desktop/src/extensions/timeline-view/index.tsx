import { useMemo, useState } from "react";
import {
  Button,
  Calendar,
  Label,
  Popover,
  PopoverContent,
  PopoverTrigger,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  ToggleGroup,
  ToggleGroupItem,
} from "@skopio/ui";
import { Group } from "@/types/tauri.gen";
import { TimelineView } from "./TimelineView";
import {
  addDays,
  differenceInMonths,
  endOfDay,
  isValid as isValidDate,
  subDays,
} from "date-fns";
import SkeletonChart from "@/components/SkeletonChart";
import { toast } from "sonner";
import { useEventFetcher } from "./hooks/useEventFetcher";

const durations = [
  { label: "15m", minutes: 15 },
  { label: "30m", minutes: 30 },
  { label: "1hr", minutes: 60 },
  { label: "2hr", minutes: 120 },
  { label: "3hr", minutes: 180 },
  { label: "4hr", minutes: 240 },
  { label: "6hr", minutes: 360 },
  { label: "12hr", minutes: 720 },
  { label: "24hr", minutes: 1440 },
  { label: "48hr", minutes: 2880 },
];

const group_options: Group[] = [
  "app",
  "branch",
  "category",
  "entity",
  "language",
  "project",
  "source",
];

const TimelineExtension = () => {
  const [group, setGroup] = useState<Group>("category");
  const [duration, setDuration] = useState<number>(15);
  const [selectedLabel, setSelectedLabel] = useState<string>(
    durations[0].label,
  );
  const [dateRange, setDateRange] = useState<{
    from: Date | null;
    to: Date | null;
  }>({
    from: new Date(),
    to: new Date(),
  });
  const isCustom = duration === 0;

  const customRange = useMemo(() => {
    if (!isCustom || !dateRange.from || !dateRange.to) return null;
    return {
      start: dateRange.from,
      end: dateRange.to,
    };
  }, [isCustom, dateRange.from, dateRange.to]);

  const { events, loading } = useEventFetcher(group, duration, customRange);

  const handleApplyCustomRange = () => {
    const { from, to } = dateRange;

    if (!from || !to || !isValidDate(from) || !isValidDate(to)) {
      toast.error("Please select valid start and end dates");
      return;
    }

    if (from >= to) {
      toast.warning("End date must be after start date");
      return;
    }
    if (differenceInMonths(addDays(to, 1), from) > 1) {
      toast.warning("The selected range cannot exceed one month");
      return;
    }

    setDuration(0);
    setSelectedLabel("");
  };

  return (
    <div className="flex-col items-center h-full w-full space-y-4 px-4">
      <h3 className="font-semibold text-3xl">Timeline</h3>
      <div className="flex flex-wrap justify-start gap-2">
        <Label htmlFor="timePreset" className="text-neutral-800">
          Show last
        </Label>
        <ToggleGroup
          type="single"
          variant="outline"
          size="default"
          id="timePreset"
          value={selectedLabel}
          onValueChange={(val) => {
            if (!val) return;
            setSelectedLabel(val);
            const selected = durations.find((d) => d.label === val);
            if (selected) {
              setSelectedLabel(val);
              setDuration(selected.minutes);
            }
          }}
        >
          {durations.map((d) => (
            <ToggleGroupItem
              key={d.label}
              value={d.label}
              className="hover:cursor-pointer"
            >
              {d.label}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </div>

      <div className="flex flex-wrap justify-start items-center gap-2 mt-4">
        <Label htmlFor="date-from" className="text-neutral-800">
          Show from
        </Label>
        <Popover>
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              id="date-from"
              className="w-30 justify-between font-normal"
            >
              {dateRange.from
                ? dateRange.from.toLocaleDateString("en-US", {
                    day: "2-digit",
                    month: "short",
                    year: "numeric",
                  })
                : "Select date"}
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-auto overflow-hidden p-0" align="start">
            <Calendar
              mode="single"
              selected={dateRange.from ?? undefined}
              captionLayout="dropdown"
              onSelect={(date) =>
                setDateRange((prev) => ({ ...prev, from: date ?? null }))
              }
            />
          </PopoverContent>
        </Popover>
        <Label htmlFor="date-to" className="text-gray-700">
          to
        </Label>
        <Popover>
          <PopoverTrigger asChild>
            <Button
              variant="secondary"
              id="date-to"
              className="w-30 justify-between font-normal"
            >
              {dateRange.to
                ? dateRange.to.toLocaleDateString("en-US", {
                    day: "2-digit",
                    month: "short",
                    year: "numeric",
                  })
                : "Select date"}
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-auto overflow-hidden p-0" align="start">
            <Calendar
              mode="single"
              selected={dateRange.to ?? undefined}
              captionLayout="dropdown"
              onSelect={(date) =>
                setDateRange((prev) => ({ ...prev, to: date ?? null }))
              }
              disabled={dateRange.from ? { before: dateRange.from } : undefined}
            />
          </PopoverContent>
        </Popover>
        <Button
          variant="secondary"
          onClick={handleApplyCustomRange}
          className="px-4 py-2 rounded font-medium border"
        >
          Apply
        </Button>
      </div>

      <div className="flex flex-wrap justify-start items-center gap-2 mt-4">
        <Label htmlFor="filter" className="text-neutral-700">
          Filter:
        </Label>
        <Select
          value={group}
          onValueChange={(value) => setGroup(value as Group)}
        >
          <SelectTrigger className="w-38" id="filter">
            <SelectValue placeholder="Select a filter" />
          </SelectTrigger>
          <SelectContent>
            {group_options.map((option) => (
              <SelectItem key={option} value={option}>
                {option}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {loading ? (
        <SkeletonChart />
      ) : (
        <TimelineView
          durationMinutes={duration}
          groupedEvents={events}
          customStart={
            customRange
              ? subDays(endOfDay(customRange?.start ?? new Date()), 1)
              : undefined
          }
          customEnd={
            customRange
              ? addDays(endOfDay(customRange?.end ?? new Date()), 1)
              : undefined
          }
        />
      )}
    </div>
  );
};

export default TimelineExtension;
