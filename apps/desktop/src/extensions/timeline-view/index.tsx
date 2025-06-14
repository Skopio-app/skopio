import { useEffect, useRef, useState } from "react";
import { TimelineView } from "./TimelineView";

export type StreamableEvent = {
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

export type AfkEventStream = {
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
  const [eventData, setEventData] = useState<StreamableEvent[]>([]);
  const [afkEventData, setAfkEventData] = useState<AfkEventStream[]>([]);
  const [durationMinutes, setDurationMinutes] = useState<number>(15);

  const eventSocketRef = useRef<WebSocket | null>(null);
  const afkSocketRef = useRef<WebSocket | null>(null);

  const openEventStream = (minutes: number) => {
    eventSocketRef.current?.close();
    const socket = new WebSocket("ws://localhost:8080/ws/events");

    socket.onopen = () => {
      console.log("Event WebSocket connected");
      socket.send(
        JSON.stringify({
          type: "duration_request",
          minutes,
        }),
      );
    };

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as StreamableEvent[];
        console.log("Received events:", data);
        setEventData(data);
      } catch (e) {
        console.error("Failed to parse event stream:", e);
      }
    };

    socket.onerror = (err) => {
      console.error("WebSocket error:", err);
    };

    socket.onclose = () => {
      console.warn("Event WebSocket closed");
    };

    eventSocketRef.current = socket;
  };

  const openAfkStream = (minutes: number) => {
    afkSocketRef.current?.close();
    const socket = new WebSocket("ws://localhost:8080/ws/afk");

    socket.onopen = () => {
      console.log("AFK event WebSocket connected");
      socket.send(
        JSON.stringify({
          type: "duration_request",
          minutes,
        }),
      );
    };

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as AfkEventStream[];
        console.log("Received AFK events: ", data);
        setAfkEventData(data);
      } catch (e) {
        console.error("Failed to parse AFK event stream: ", e);
      }
    };

    socket.onerror = (err) => {
      console.error("WebSocket error: ", err);
    };

    socket.onclose = () => {
      console.log("AFK event WebSocket closed");
    };

    afkSocketRef.current = socket;
  };

  useEffect(() => {
    openEventStream(durationMinutes);
    openAfkStream(durationMinutes);

    return () => {
      eventSocketRef.current?.close();
      afkSocketRef.current?.close();
    };
  }, [durationMinutes]);

  return (
    <div className="flex-col tems-center space-y-4 px-4 py-20">
      <div className="flex flex-wrap justify-center gap-2">
        {durations.map((d) => (
          <button
            key={d.minutes}
            className={`px-3 py-1 rounded text-sm font-medium border transition ${
              durationMinutes === d.minutes
                ? "bg-blue-600 text-white border-blue-700"
                : "bg-white text-gray-800 border-gray-300 hover:bg-gray-100"
            }`}
            onClick={() => setDurationMinutes(d.minutes)}
          >
            {d.label}
          </button>
        ))}
      </div>
      <TimelineView
        afkEventStream={afkEventData}
        eventStream={eventData}
        durationMinutes={durationMinutes}
      />
    </div>
  );
};

export default TimelineExtension;
