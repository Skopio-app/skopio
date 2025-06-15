import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { TimelineView } from "./TimelineView";
import {
  endOfDay,
  formatISO,
  parseISO,
  startOfDay,
  isValid as isValidDate,
  addDays,
  differenceInMonths,
} from "date-fns";

export type EventStream = {
  id: number;
  timestamp: string;
  endTimestamp?: string | null;
  duration?: number | null;
  activity_type: string;
  app?: string | null;
  entity?: string | null;
  project?: string | null;
  branch?: string | null;
  language?: string | null;
};

export type AFKEventStream = {
  id: number;
  afk_start: string;
  afk_end: string | null;
  duration: number;
};

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
  const [eventDataMap, setEventDataMap] = useState<Map<string, EventStream>>(
    new Map(),
  );
  const [afkEventDataMap, setAfkEventDataMap] = useState<
    Map<string, AFKEventStream>
  >(new Map());
  const [currentDurationMinutes, setCurrentDurationMinutes] =
    useState<number>(15);

  const [customStartDate, setCustomStartDate] = useState<string>("");
  const [customEndDate, setCustomEndDate] = useState<string>("");

  const [queriedInterval, setQueriedInterval] = useState<
    [Date, Date] | undefined
  >(undefined);
  // const [showQueriedInterval, setShowQueriedInterval] = useState<boolean>(false);

  const eventSocketRef = useRef<WebSocket | null>(null);
  const afkSocketRef = useRef<WebSocket | null>(null);

  const updateMapState = useCallback(
    (
      prevMap: Map<string, any>,
      newData: any[],
      idKey: string | ((item: any) => string),
    ) => {
      const newMap = new Map(prevMap);
      newData.forEach((item) => {
        const id =
          typeof idKey === "function" ? idKey(item) : String(item[idKey]);
        newMap.set(id, item);
      });
      return newMap;
    },
    [],
  );

  const requestDataForRange = useCallback((start: Date, end: Date) => {
    console.log(
      `Requesting data for range: ${formatISO(start)} to ${formatISO(end)}`,
    );

    if (eventSocketRef.current?.readyState === WebSocket.OPEN) {
      eventSocketRef.current.send(
        JSON.stringify({
          type: "range_request",
          start_timestamp: formatISO(start),
          end_timestamp: formatISO(end),
        }),
      );
    } else {
      console.warn("Event WebSocket not open to send range request.");
    }

    if (afkSocketRef.current?.readyState === WebSocket.OPEN) {
      afkSocketRef.current.send(
        JSON.stringify({
          type: "range_request",
          start_timestamp: formatISO(start),
          end_timestamp: formatISO(end),
        }),
      );
    } else {
      console.warn("AFK WebSocket not open to range request");
    }
  }, []);

  const setupWebSocket = useCallback(
    (
      url: string,
      onMessage: (data: any) => void,
      socketRef: React.MutableRefObject<WebSocket | null>,
    ) => {
      const currentSocketInstance = socketRef.current;
      if (currentSocketInstance) {
        currentSocketInstance.close();
      }

      const socket = new WebSocket(url);

      socket.onopen = () => {
        console.log(`${url} WebSocket connected`);

        socket.send(
          JSON.stringify({
            type: "duration_request",
            minutes: currentDurationMinutes,
          }),
        );
      };
      socket.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          onMessage(data);
        } catch (err) {
          console.error(`Failed to parse WebSocket message from ${url}:`, err);
        }
      };

      socket.onerror = (err) => {
        console.error(`WebSocket error for ${url}: `, err);
      };

      socket.onclose = () => {
        console.warn(`${url} WebSocket closed`);
      };

      socketRef.current = socket;

      return () => {
        console.log(`Cleaning up ${url} socket: `, socket);
        socket.close();
      };
    },
    [currentDurationMinutes],
  );

  useEffect(() => {
    const cleanupEventSocket = setupWebSocket(
      "ws://localhost:8080/ws/events",
      (data: EventStream[]) => {
        setEventDataMap((prevMap) => updateMapState(prevMap, data, "id"));
      },
      eventSocketRef,
    );

    const cleanupAFKEventSocket = setupWebSocket(
      "ws://localhost:8080/ws/afk",
      (data: AFKEventStream[]) => {
        setAfkEventDataMap((prevMap) => updateMapState(prevMap, data, "id"));
      },
      afkSocketRef,
    );

    return () => {
      cleanupEventSocket();
      cleanupAFKEventSocket();
    };
  }, [setupWebSocket, updateMapState]);

  const handleApplyCustomRange = useCallback(() => {
    const start = customStartDate
      ? startOfDay(parseISO(customStartDate))
      : null;
    const end = customEndDate ? endOfDay(parseISO(customEndDate)) : null;

    if (!start || !end || !isValidDate(start) || !isValidDate(end)) {
      alert("Please select valid end and start dates");
      return;
    }

    if (start >= end) {
      alert("End date must be after start date");
      return;
    }

    const adjustedEnd = addDays(end, 1);
    if (differenceInMonths(adjustedEnd, start) > 1) {
      alert("The selected duration cannot be more than one month");
      return;
    }

    setQueriedInterval([start, end]);
    // setShowQueriedInterval(true);
    requestDataForRange(start, end);

    setCurrentDurationMinutes(0);
  }, [customStartDate, customEndDate, requestDataForRange]);

  useEffect(() => {
    if (currentDurationMinutes > 0) {
      // setShowQueriedInterval(false);
      setQueriedInterval(undefined);
      const now = new Date();
      const initialMax = new Date(now.getTime() + 5 * 60 * 1000);
      const initialMin = new Date(
        initialMax.getTime() - currentDurationMinutes * 60 * 1000,
      );
      requestDataForRange(initialMin, initialMax);
    }
  }, [currentDurationMinutes, requestDataForRange]);

  const eventDataArray = useMemo(
    () => Array.from(eventDataMap.values()),
    [eventDataMap],
  );
  const afkEventDataArray = useMemo(
    () => Array.from(afkEventDataMap.values()),
    [afkEventDataMap],
  );

  return (
    <div className="flex-col items-center h-full w-full space-y-4 px-4 py-20">
      <div className="flex flex-wrap justify-center gap-2">
        {durations.map((d) => (
          <button
            key={d.minutes}
            className={`px-3 py-1 rounded text-sm font-medium border transition ${
              currentDurationMinutes === d.minutes
                ? "bg-blue-600 text-white border-blue-700"
                : "bg-white text-gray-800 border-gray-300 hover:bg-gray-100"
            }`}
            onClick={() => setCurrentDurationMinutes(d.minutes)}
          >
            {d.label}
          </button>
        ))}
      </div>

      <div className="flex flex-wrap justify-center items-center gap-2 mt-4">
        <label htmlFor="startDate" className="text-gray-700">
          Show from
        </label>
        <input
          type="date"
          id="startDate"
          value={customStartDate}
          onChange={(e) => setCustomStartDate(e.target.value)}
          className="p-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <label htmlFor="endDate" className="text-gray-700">
          to
        </label>
        <input
          type="date"
          id="endDate"
          value={customEndDate}
          onChange={(e) => setCustomEndDate(e.target.value)}
          className="p-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          onClick={handleApplyCustomRange}
          className="px-4 py-2 bg-green-400 text-neutral-200 rounded font-medium border border-green-700 hover:bg-green-700 transition"
        >
          Apply
        </button>
      </div>
      <TimelineView
        afkEventStream={afkEventDataArray}
        eventStream={eventDataArray}
        durationMinutes={currentDurationMinutes}
        requestDataForRange={requestDataForRange}
        queriedInterval={queriedInterval}
        // showQueriedInterval={showQueriedInterval}
      />
    </div>
  );
};

export default TimelineExtension;
