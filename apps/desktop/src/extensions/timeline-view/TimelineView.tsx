import React, { useEffect, useRef, useMemo } from "react";
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
import { AfkEventStream, StreamableEvent } from "./index";

type Props = {
  afkEventStream: AfkEventStream[];
  eventStream: StreamableEvent[];
  queriedInterval?: [Date, Date];
  showQueriedInterval?: boolean;
  durationMinutes: number;
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
  afkEventStream,
  eventStream,
  queriedInterval,
  showQueriedInterval = true,
  durationMinutes,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const timelineRef = useRef<Timeline | null>(null);
  const dataSetRef = useRef<DataSet<any> | null>(null);

  const groups = useMemo(
    () => [
      { id: "AFK", content: "AFK" },
      { id: "VSCode", content: "skopio-vscode" },
      { id: "General", content: "skopio-desktop" },
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

  const getTimelineRange = (
    dataItems: TimelineDataItem[],
    currentDurationMinutes: number,
  ): { min: Date; max: Date } => {
    const allStarts = dataItems.map((e) => e.start).filter(Boolean);
    const allEnds = dataItems.map((e) => e.end).filter(Boolean);

    const now = new Date();

    if (!allStarts.length || !allEnds.length) {
      const max = new Date(now.getTime() + 2 * 60 * 1000);
      const min = new Date(max.getTime() - currentDurationMinutes * 60 * 1000);
      return { min, max };
    }

    const earliest = new Date(Math.min(...allStarts.map((d) => d.getTime())));
    const latest = new Date(Math.max(...allEnds.map((d) => d.getTime())));

    const calculatedMin = new Date(
      latest.getTime() - currentDurationMinutes * 60 * 1000,
    );
    const min = calculatedMin < earliest ? earliest : calculatedMin;
    const max = new Date(min.getTime() + currentDurationMinutes * 60 * 1000);

    return { min, max };
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

    dataSetRef.current = new DataSet();
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

    return () => {
      if (timelineRef.current) {
        timelineRef.current.destroy();
        timelineRef.current = null;
      }

      if (dataSetRef.current) {
        dataSetRef.current.clear();
        dataSetRef.current = null;
      }
    };
  }, [groups, durationMinutes]);

  useEffect(() => {
    if (!dataSetRef.current || !timelineRef.current) {
      console.warn(
        "Timeline or DataSet not initialized, skipping data update.",
      );
      return;
    }

    const itemsToUpdateOrAdd: any[] = [];
    const itemsToRemoveIds: string[] = [];
    const currentDataSetIds = new Set(dataSetRef.current.getIds());

    const eventStreamItemIds = new Set<string>();
    for (const e of eventStream.values()) {
      const start = safeParseISO(e.timestamp);
      const end = e.endTimestamp
        ? safeParseISO(e.endTimestamp)
        : addSeconds(start ?? 0, e.duration ?? 0);

      if (!start || !end || differenceInSeconds(end, start) <= 1) continue;

      const id = String(e.id);
      eventStreamItemIds.add(id);

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

      if (currentDataSetIds.has(id)) {
        dataSetRef.current.update(item);
      } else {
        itemsToUpdateOrAdd.push(item);
      }
    }

    const afkEventStreamItemIds = new Set<string>();
    for (const e of afkEventStream.values()) {
      const start = safeParseISO(e.afk_start);
      const end = e.afk_end
        ? safeParseISO(e.afk_end)
        : addSeconds(start ?? 0, e.duration ?? 0);
      if (!start || !end || isNaN(start.getTime()) || isNaN(end.getTime()))
        continue;

      const id = `afk-${e.id}`;
      afkEventStreamItemIds.add(id);

      const item: TimelineDataItem = {
        id,
        group: "AFK",
        content: "AFK",
        start,
        end,
        title: `AFK for ${e.duration} seconds`,
        style: "background-color: #868e96; border-color: #495057;",
      };

      if (currentDataSetIds.has(id)) {
        dataSetRef.current.update(item);
      } else {
        itemsToUpdateOrAdd.push(item);
      }
    }

    const queriedIntervalId = "query-interval";
    if (queriedInterval && showQueriedInterval) {
      const [start, end] = queriedInterval;
      const queryItem: TimelineDataItem = {
        id: queriedIntervalId,
        group: "Query",
        content: "Query Range",
        title: "Queried Time Interval",
        start,
        end,
        style: "background-color: #aaa; height: 10px;",
      };
      if (currentDataSetIds.has(queriedIntervalId)) {
        dataSetRef.current.update(queryItem);
      } else {
        itemsToUpdateOrAdd.push(queryItem);
      }
    } else {
      if (currentDataSetIds.has(queriedIntervalId)) {
        itemsToRemoveIds.push(queriedIntervalId);
      }
    }

    dataSetRef.current.getIds().forEach((id: any) => {
      if (id === queriedIntervalId && queriedInterval && showQueriedInterval) {
        return;
      }
      if (!eventStreamItemIds.has(id) && !afkEventStreamItemIds.has(id)) {
        itemsToRemoveIds.push(id);
      }
    });

    if (itemsToUpdateOrAdd.length > 0) {
      dataSetRef.current.add(itemsToUpdateOrAdd);
    }
    if (itemsToRemoveIds.length > 0) {
      dataSetRef.current.remove(itemsToRemoveIds);
    }

    const allItemsInDataSet = dataSetRef.current.get() as TimelineDataItem[];
    const viewRange = getTimelineRange(allItemsInDataSet, durationMinutes);

    if (viewRange) {
      timelineRef.current.setOptions({
        min: viewRange.min,
        max: viewRange.max,
      });

      const latestOverallEnd = _.maxBy(allItemsInDataSet, "end")?.end ?? null;
      if (latestOverallEnd) {
        timelineRef.current.moveTo(latestOverallEnd, { animation: true });
      } else {
        timelineRef.current.moveTo(viewRange.max, { animation: true });
      }
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
      timelineRef.current.moveTo(initialMax, { animation: true });
    }
  }, [
    eventStream,
    afkEventStream,
    queriedInterval,
    showQueriedInterval,
    durationMinutes,
  ]);

  return (
    <div className="flex justify-center px-4">
      <div
        ref={containerRef}
        id="visualization"
        className="w-full max-w-7xl my-10"
      />
    </div>
  );
};
