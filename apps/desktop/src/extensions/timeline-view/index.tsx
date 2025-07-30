import { useCallback, useEffect, useState } from "react";
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
import {
  BucketedSummaryInput,
  commands,
  EventGroup,
  EventGroupResult,
  Group,
} from "../../types/tauri.gen";
import { useGroupFilter } from "./stores/useGroupFilter";
import { TimelineView } from "./TimelineView";
import {
  // addDays,
  // differenceInMonths,
  // endOfDay,
  // parseISO,
  // startOfDay,
  // isValid as isValidDate,
  formatISO,
} from "date-fns";

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

const TimelineExtension = () => {
  const [currentDurationMinutes, setCurrentDurationMinutes] =
    useState<number>(15);
  const [selectedLabel, setSelectedLabel] = useState<string>(
    durations[0].label,
  );
  const [openTo, setOpenTo] = useState<boolean>(false);
  const [openFrom, setOpenFrom] = useState<boolean>(false);
  const [dateFrom, setDateFrom] = useState<Date | undefined>(new Date());
  const [dateTo, setDateTo] = useState<Date | undefined>(new Date());
  const [events, setEvents] = useState<EventGroup[]>([]);
  const { group } = useGroupFilter();

  const group_options: Group[] = [
    "app",
    "branch",
    "category",
    "entity",
    "language",
    "project",
  ];

  useEffect(() => {
    const fetchEventData = async () => {
      const query: BucketedSummaryInput = {
        preset: { lastNMinutes: currentDurationMinutes },
        group_by: group,
        include_afk: false,
      };

      try {
        const result: EventGroupResult = await commands.fetchEvents(query);
        if ("Grouped" in result) {
          setEvents(result.Grouped);
          console.log("Thr result: ", result.Grouped);
        }
      } catch (err) {
        console.error("Failed to fetch events: ", err);
      }
    };

    fetchEventData();
  }, [currentDurationMinutes, group]);

  // const handleApplyCustomRange = useCallback(() => {
  //   const start = dateFrom
  //     ? startOfDay(parseISO(dateFrom.toISOString()))
  //     : null;
  //   const end = dateTo ? endOfDay(parseISO(dateTo.toISOString())) : null;

  //   if (!start || !end || !isValidDate(start) || !isValidDate(end)) {
  //     alert("Please select valid end and start dates");
  //     return;
  //   }

  //   if (start >= end) {
  //     alert("End date must be after start date");
  //     return;
  //   }

  //   const adjustedEnd = addDays(end, 1);
  //   if (differenceInMonths(adjustedEnd, start) > 1) {
  //     alert("The selected duration cannot be more than one month");
  //     return;
  //   }

  //   setCurrentDurationMinutes(0);
  // }, [dateTo, dateFrom]);

  const requestDataForRange = useCallback((start: Date, end: Date) => {
    console.log(
      `Requesting data for range: ${formatISO(start)} to ${formatISO(end)}`,
    );
  }, []);

  return (
    <div className="flex-col items-center h-full w-full space-y-4 px-4 py-8">
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
              setCurrentDurationMinutes(selected.minutes);
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
        <Popover open={openFrom} onOpenChange={setOpenFrom}>
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              id="date-from"
              className="w-30 justify-between font-normal"
            >
              {dateFrom
                ? dateFrom.toLocaleDateString("en-US", {
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
              selected={dateFrom}
              captionLayout="dropdown"
              onSelect={(date) => {
                setDateFrom(date);
                setOpenFrom(false);
              }}
            />
          </PopoverContent>
        </Popover>
        <Label htmlFor="date-to" className="text-gray-700">
          to
        </Label>
        <Popover open={openTo} onOpenChange={setOpenTo}>
          <PopoverTrigger asChild>
            <Button
              variant="outline"
              id="date-to"
              className="w-30 justify-between font-normal"
            >
              {dateTo
                ? dateTo.toLocaleDateString("en-US", {
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
              selected={dateTo}
              captionLayout="dropdown"
              onSelect={(date) => {
                setDateTo(date);
                setOpenTo(false);
              }}
              disabled={dateFrom && { before: dateFrom }}
            />
          </PopoverContent>
        </Popover>
        {/* <Button
          variant="outline"
          onClick={handleApplyCustomRange}
          className="px-4 py-2 rounded font-medium border"
        >
          Apply
        </Button> */}
      </div>

      <div className="flex flex-wrap justify-start items-center gap-2 mt-4">
        <Label htmlFor="filter" className="text-neutral-700">
          Filter:
        </Label>
        <Select
          value={group}
          onValueChange={(value) =>
            useGroupFilter.setState({ group: value as Group })
          }
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

      <TimelineView
        durationMinutes={currentDurationMinutes}
        requestDataForRange={requestDataForRange}
        groupedEvents={events}
      />
    </div>
  );
};

export default TimelineExtension;
