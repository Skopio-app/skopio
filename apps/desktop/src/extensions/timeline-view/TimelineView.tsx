import React, { useEffect, useRef, useMemo, useCallback } from "react";
import { Timeline, TimelineOptions, DataSet } from "vis-timeline/standalone";
import Color from "color";
import _ from "lodash";
import {
  parseISO,
  addSeconds,
  differenceInSeconds,
  isValid as isValidDate,
  format,
} from "date-fns";

import "vis-timeline/styles/vis-timeline-graph2d.css";
import { formatDuration } from "../../utils/time";
import { EventGroup, FullEvent } from "../../types/tauri.gen";

type Props = {
  requestDataForRange: (start: Date, end: Date) => void;
  durationMinutes: number;
  groupedEvents: EventGroup[];
};

interface TimelineDataItem {
  id?: string | number;
  start: Date | number | string;
  end?: Date | number | string;
  group?: any;
  content: string;
  title?: string;
  style?: string;
  subgroup?: string | number;
  type?: string;
  limitSize?: boolean;
  editable?: boolean | object;
  className?: string;
  align?: string;
  selectable?: boolean;
}

interface TimelineGroup {
  id: string | number;
  title?: string;
  className?: string;
  content?: string | HTMLElement;
  style?: string;
  subgroupOrder?: string | ((a: any, b: any) => number);
  subgroupStack?: Record<string, boolean> | boolean;
  subgroupVisibility?: Record<string, boolean>;
  visible?: boolean;
  nestedGroups?: Array<string | number>;
  showNested?: boolean;
}

export const TimelineView: React.FC<Props> = ({
  // requestDataForRange,
  durationMinutes,
  groupedEvents,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const timelineRef = useRef<Timeline | null>(null);
  const dataSetRef = useRef<DataSet<TimelineDataItem> | null>(null);
  const animationTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // const FETCH_BUFFER_MINUTES = 30; // Fetch 30 mins before and after the visible range
  const PRUNE_BUFFER_MINUTES = 60; // Prune items 60 minutes outside the visible range
  const ANIMATION_INACTIVITY_DELAY_MS = 5000; // Animate to latest after 10 seconds

  const groups: TimelineGroup[] = useMemo(() => {
    const dynamicGroups: TimelineGroup[] = [];
    groupedEvents.forEach((group) => {
      dynamicGroups.push({ id: group.group, content: group.group });
    });

    return dynamicGroups;
  }, [groupedEvents]);

  const safeParseISO = (dateInput: unknown): Date | null => {
    if (typeof dateInput === "number") {
      if (dateInput > 1e12) return new Date(dateInput); // ms
      if (dateInput > 1e9) return new Date(dateInput * 1000); // s
      return null;
    }

    if (typeof dateInput === "string") {
      const trimmed = dateInput.replace(/\.(\d{3})\d+/, ".$1");

      try {
        const parsed = parseISO(trimmed);
        return isValidDate(parsed) ? parsed : null;
      } catch {
        return null;
      }
    }

    return null;
  };

  const getColorForActivity = (type: string): string => {
    const colors: Record<string, string> = {
      Browsing: "#4dabf7",
      "Code Reviewing": "#63e6be",
      Coding: "#5d8b14",
      Debugging: "#ffd43b",
      Testing: "#f783ac",
    };
    return colors[type] || "#dbe4ed";
  };

  const clearAnimationTimeout = useCallback(() => {
    if (animationTimeoutRef.current) {
      clearTimeout(animationTimeoutRef.current);
      animationTimeoutRef.current = null;
    }
  }, []);

  const handleRangeChanged = useMemo(() => {
    return _.debounce(({ start, end }: { start: Date; end: Date }) => {
      if (!dataSetRef.current || !timelineRef.current) return;

      console.log(
        `Range changed: ${start.toISOString()} to ${end.toISOString()}`,
      );

      clearAnimationTimeout();

      // Request new data for the expanded range (visible range + buffer)
      // const fetchStart = new Date(
      //   start.getTime() - FETCH_BUFFER_MINUTES * 60 * 1000
      // );
      // const fetchEnd = new Date(
      //   end.getTime() + FETCH_BUFFER_MINUTES * 60 * 1000
      // );
      // requestDataForRange(fetchStart, fetchEnd);

      // Prune data outside a larger buffer from the DataSet
      const pruneStart = new Date(
        start.getTime() - PRUNE_BUFFER_MINUTES * 60 * 1000,
      );
      const pruneEnd = new Date(
        end.getTime() + PRUNE_BUFFER_MINUTES * 60 * 1000,
      );

      const itemsToRemove: string[] = [];
      dataSetRef.current.forEach((item: any) => {
        // Remove items whose entire range is outside the prune buffer
        if (item.end < pruneStart || item.start > pruneEnd) {
          itemsToRemove.push(item.id);
        }
      });

      if (itemsToRemove.length > 0) {
        console.log(`Pruning ${itemsToRemove.length} items from DataSet`);
        dataSetRef.current.remove(itemsToRemove);
      }
    }, 500);
  }, [
    // requestDataForRange,
    // FETCH_BUFFER_MINUTES,
    PRUNE_BUFFER_MINUTES,
    clearAnimationTimeout,
  ]);

  const formatTimelineTitle = ({
    start,
    end,
    duration,
    app,
    entity,
  }: {
    start: Date;
    end: Date;
    duration: number;
    app?: string | null;
    entity?: string | null;
  }): string => {
    const formattedDuration = formatDuration(duration);

    return [
      `Start: ${format(start, "HH:mm:ss")}`,
      `End: ${format(end, "HH:mm:ss")}`,
      `Duration: ${formattedDuration}`,
      `App: ${app ?? "unknown"}`,
      `Entity: ${entity ?? "unknown"}`,
    ]
      .map((line) => line.replace(/"/g, "&quot;"))
      .join("<br/>");
  };

  useEffect(() => {
    if (!containerRef.current) return;

    if (timelineRef.current) {
      timelineRef.current.destroy();
      timelineRef.current = null;
    }

    if (dataSetRef.current) {
      dataSetRef.current.clear();
      dataSetRef.current = null;
    }

    const options: TimelineOptions = {
      zoomMin: 60_000,
      zoomMax: 1000 * 60 * 60 * 24 * 31 * 3, // 3 months
      stack: false,
      showMinorLabels: true,
      tooltip: {
        followMouse: true,
        overflowMethod: "cap",
        delay: 0,
      },
    };

    dataSetRef.current = new DataSet<TimelineDataItem>();
    timelineRef.current = new Timeline(
      containerRef.current,
      dataSetRef.current,
      groups,
      options,
    );

    const now = new Date();
    const initialMax = new Date(now.getTime() + 5 * 60 * 1000);
    const initialMin = new Date(
      initialMax.getTime() - durationMinutes * 60 * 1000,
    );

    timelineRef.current.setOptions({
      min: initialMin,
      max: initialMax,
      start: initialMin,
      end: initialMax,
    });

    timelineRef.current.on("rangechanged", handleRangeChanged);

    // Initial data request for the default view
    // requestDataForRange(initialMin, initialMax);

    return () => {
      if (timelineRef.current) {
        timelineRef.current.off("rangechanged", handleRangeChanged);
        handleRangeChanged.cancel();
        timelineRef.current.destroy();
        timelineRef.current = null;
      }

      if (dataSetRef.current) {
        dataSetRef.current.clear();
        dataSetRef.current = null;
      }

      clearAnimationTimeout();
    };
  }, [
    groups,
    durationMinutes,
    handleRangeChanged,
    // requestDataForRange,
    clearAnimationTimeout,
  ]);

  useEffect(() => {
    if (!dataSetRef.current || !timelineRef.current) {
      console.warn(
        "Timeline or DataSet not initialized, skipping data update.",
      );
      return;
    }

    const itemsToProcess: TimelineDataItem[] = [];

    groupedEvents.forEach((groupData: EventGroup) => {
      groupData.events.forEach((e: FullEvent) => {
        const start = safeParseISO(e.timestamp);
        const end = e.endTimestamp
          ? safeParseISO(e.endTimestamp)
          : addSeconds(start ?? 0, e.duration ?? 0);

        if (!start || !end || differenceInSeconds(end, start) <= 1) return;

        const id = String(e.id);
        const group = groupData.group;
        const color = getColorForActivity(e.category);

        const item: TimelineDataItem = {
          id,
          group,
          content: `${e.app ?? "unknown"}`,
          start,
          end,
          title: formatTimelineTitle({
            start,
            end,
            app: e.app,
            entity: e.entity,
            duration: e.duration ?? 0,
          }),
          style: `background-color: ${color}; border-color: ${Color(color).darken(0.6)};`,
        };

        itemsToProcess.push(item);
      });
    });

    if (itemsToProcess.length > 0) {
      dataSetRef.current.update(itemsToProcess);
    }

    const allItemsInDataSet = dataSetRef.current.get() as TimelineDataItem[];
    const latestOverallEnd = _.maxBy(allItemsInDataSet, "end")?.end ?? null;

    if (latestOverallEnd) {
      const currentRange = timelineRef.current.getWindow();

      const isLatestOutsideView =
        latestOverallEnd > currentRange.end ||
        latestOverallEnd < currentRange.start;
      if (isLatestOutsideView) {
        clearAnimationTimeout();

        animationTimeoutRef.current = setTimeout(() => {
          if (timelineRef.current) {
            console.log("Animating to latest entry after inactivity.");
            timelineRef.current.moveTo(latestOverallEnd, { animation: true });
          }
        }, ANIMATION_INACTIVITY_DELAY_MS);
      }
    }
  }, [groupedEvents, clearAnimationTimeout]);

  return (
    <div className="flex justify-center items-center">
      <div
        ref={containerRef}
        id="visualization"
        className="w-full max-w-8xl my-10"
      />
    </div>
  );
};
