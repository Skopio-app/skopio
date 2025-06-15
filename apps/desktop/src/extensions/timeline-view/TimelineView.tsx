import React, { useEffect, useRef, useMemo, useCallback } from "react";
import { Timeline, TimelineOptions, DataSet } from "vis-timeline/standalone";
import Color from "color";
import _ from "lodash";
import {
  parseISO,
  addSeconds,
  differenceInSeconds,
  isValid as isValidDate,
} from "date-fns";

import "vis-timeline/styles/vis-timeline-graph2d.css";
import { AFKEventStream, EventStream } from "./index";

type Props = {
  requestDataForRange: (start: Date, end: Date) => void;
  afkEventStream: AFKEventStream[];
  eventStream: EventStream[];
  durationMinutes: number;
  queriedInterval?: [Date, Date];
  // showQueriedInterval?: boolean;
};

// TODO: Check for other properties to add.
interface TimelineDataItem {
  id: string;
  start: Date;
  end: Date;
  group: string;
  content: string;
  title: string;
  style: string;
}

export const TimelineView: React.FC<Props> = ({
  requestDataForRange,
  afkEventStream,
  eventStream,
  durationMinutes,
  queriedInterval,
  // showQueriedInterval = false,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const timelineRef = useRef<Timeline | null>(null);
  const dataSetRef = useRef<DataSet<TimelineDataItem> | null>(null);
  const animationTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const FETCH_BUFFER_MINUTES = 30; // Fetch 30 mins before and after the visible range
  const PRUNE_BUFFER_MINUTES = 60; // Prune items 60 minutes outside the visible range
  const ANIMATION_INACTIVITY_DELAY_MS = 5000; // Animate to latest after 10 seconds

  const groups = useMemo(
    () => [
      { id: "AFK", content: "AFK" },
      { id: "VSCode", content: "skopio-vscode" },
      { id: "General", content: "skopio-desktop" },
      // { id: "Query", content: "Query" },
    ],
    [],
  );

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

  const getGroupId = (app?: string | null): string => {
    if (!app) return "General";
    const name = app.toLowerCase();
    return name === "vscode" || name === "code" ? "VSCode" : "General";
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
      const fetchStart = new Date(
        start.getTime() - FETCH_BUFFER_MINUTES * 60 * 1000,
      );
      const fetchEnd = new Date(
        end.getTime() + FETCH_BUFFER_MINUTES * 60 * 1000,
      );
      requestDataForRange(fetchStart, fetchEnd);

      // Prune data outside a larger buffer from the DataSet
      const pruneStart = new Date(
        start.getTime() - PRUNE_BUFFER_MINUTES * 60 * 1000,
      );
      const pruneEnd = new Date(
        end.getTime() + PRUNE_BUFFER_MINUTES * 60 * 1000,
      );

      const itemsToRemove: string[] = [];
      dataSetRef.current.forEach((item: any) => {
        // if (
        //   item.id === "query-interval" &&
        //   queriedInterval &&
        //   showQueriedInterval
        // ) {
        //   return;
        // }
        // Remove items whose entire range is outside the prune buffer
        if (item.end < pruneStart || item.start > pruneEnd) {
          itemsToRemove.push(item.id);
        }
      });

      if (itemsToRemove.length > 0) {
        console.log(`Pruning ${itemsToRemove.length} items from DataSet`);
        dataSetRef.current.remove(itemsToRemove);
      }
    }, 500); // debounce for 500ms
  }, [
    requestDataForRange,
    FETCH_BUFFER_MINUTES,
    PRUNE_BUFFER_MINUTES,
    clearAnimationTimeout,
    // queriedInterval,
    // showQueriedInterval,
  ]);

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

    let initialMin: Date = new Date();
    let initialMax: Date = new Date();

    if (queriedInterval) {
      initialMin = queriedInterval[0];
      initialMax = queriedInterval[1];

      const buffer = (initialMax.getTime() - initialMin.getTime()) * 0.1;
      timelineRef.current.setOptions({
        min: new Date(initialMin.getTime() - buffer),
        max: new Date(initialMax.getTime() + buffer),
        start: new Date(initialMin.getTime() - buffer),
        end: new Date(initialMax.getTime() + buffer),
      });
    } else {
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
    }

    timelineRef.current.on("rangechanged", handleRangeChanged);

    // Initial data request for the default view
    requestDataForRange(initialMin, initialMax);

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
    requestDataForRange,
    clearAnimationTimeout,
    queriedInterval,
    // showQueriedInterval
  ]);

  useEffect(() => {
    if (!dataSetRef.current || !timelineRef.current) {
      console.warn(
        "Timeline or DataSet not initialized, skipping data update.",
      );
      return;
    }

    const itemsToProcess: TimelineDataItem[] = [];

    for (const e of eventStream.values()) {
      const start = safeParseISO(e.timestamp);
      const end = e.endTimestamp
        ? safeParseISO(e.endTimestamp)
        : addSeconds(start ?? 0, e.duration ?? 0);

      if (!start || !end || differenceInSeconds(end, start) <= 1) continue;

      const id = String(e.id);
      const group = getGroupId(e.app);
      const color = getColorForActivity(e.activity_type);

      const item: TimelineDataItem = {
        id,
        group,
        content: `${e.app ?? "unknown"}`,
        start,
        end,
        title: `App: ${e.app}<br/>Entity: ${e.entity}`,
        style: `background-color: ${color}; border-color: ${Color(color).darken(0.6)};`,
      };

      itemsToProcess.push(item);
    }

    for (const e of afkEventStream.values()) {
      const start = safeParseISO(e.afk_start);
      const end = e.afk_end
        ? safeParseISO(e.afk_end)
        : addSeconds(start ?? 0, e.duration ?? 0);
      if (!start || !end || isNaN(start.getTime()) || isNaN(end.getTime()))
        continue;

      const id = `afk-${e.id}`;

      const item: TimelineDataItem = {
        id,
        group: "AFK",
        content: "AFK",
        start,
        end,
        title: `AFK for ${e.duration} seconds`,
        style: "background-color: #868e96; border-color: #495057;",
      };
      itemsToProcess.push(item); // Add for batch update/add
    }

    // const queriedIntervalId = "query-interval";
    // if (queriedInterval && showQueriedInterval) {
    //   const [start, end] = queriedInterval;
    //   const queryItem: TimelineDataItem = {
    //     id: queriedIntervalId,
    //     group: "Query",
    //     content: "Query Range",
    //     title: "Queried Time Interval",
    //     start,
    //     end,
    //     style: "background-color: rgba(173, 216, 230, 0.3); border: 1px dashed #6495ED;",
    //   };
    //   itemsToProcess.push(queryItem);
    // } else {
    //   if (dataSetRef.current.get(queriedIntervalId)) {
    //     dataSetRef.current.remove(queriedIntervalId);
    //   }
    // }

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
  }, [eventStream, afkEventStream, clearAnimationTimeout]);

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
