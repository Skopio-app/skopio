import { useVirtualizer } from "@tanstack/react-virtual";
import { format } from "date-fns";
import React, {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";

export type TimelineBin = {
  timestamp: number;
  afk: number;
  vscode: number;
  general: number;
};

export type TimelineViewProps = {
  data: TimelineBin[];
  stream?: AsyncIterable<TimelineBin>;
};

const TIME_INTERVALS = [
  15 * 60 * 1000,
  30 * 60 * 1000,
  60 * 60 * 1000,
  2 * 60 * 60 * 1000,
  3 * 60 * 60 * 1000,
  4 * 60 * 60 * 1000,
  6 * 60 * 60 * 1000,
  12 * 60 * 60 * 1000,
  24 * 60 * 60 * 1000,
  48 * 60 * 60 * 1000,
];

const BAR_HEIGHT = 20;
const ROW_HEIGHT = 30;
const LABEL_WIDTH = 100;

const binData = (
  data: TimelineBin[],
  interval: number,
  from: number,
  to: number,
): TimelineBin[] => {
  const buckets: Record<number, TimelineBin> = {};

  for (const item of data) {
    if (item.timestamp < from || item.timestamp > to) continue;
    const bucket = Math.floor(item.timestamp / interval) * interval;
    if (!buckets[bucket]) {
      buckets[bucket] = { timestamp: bucket, afk: 0, vscode: 0, general: 0 };
    }
    buckets[bucket].afk += item.afk;
    buckets[bucket].vscode += item.vscode;
    buckets[bucket].general += item.general;
  }

  return Object.values(buckets).sort((a, b) => a.timestamp - b.timestamp);
};

const TrackRow = ({
  label,
  color,
  data,
  //   index,
  style,
}: {
  label: string;
  color: string;
  data: TimelineBin[];
  index: number;
  style: React.CSSProperties;
}) => {
  return (
    <div className="flex items-center" style={style}>
      <div className="w-[100px] text-sm text-gray-500 pr-2 text-right">
        {label}
      </div>
      <div className="flex-1 flex h-[20px]">
        {data.map((bin) => (
          <div
            key={bin.timestamp}
            title={`${format(new Date(bin.timestamp), "HH:mm")}\n${label}: ${Math.round(
              bin[label.toLowerCase() as keyof TimelineBin] * 100,
            )}`}
            style={{
              width: 4,
              height: BAR_HEIGHT,
              backgroundColor: color,
              opacity: bin[label.toLowerCase() as keyof TimelineBin],
            }}
            className="hover:opacity-100 opacity-70 transition-opacity duration-150"
          ></div>
        ))}
      </div>
    </div>
  );
};

const TimeAxis = ({ bins }: { bins: TimelineBin[] }) => {
  const labelWidth = 40; // px
  const step = Math.ceil(labelWidth / 4); // Show every Nth bin depending on zoom

  return (
    <div
      className="flex items-end text-[10px] text-gray-500 pl-[100px] h-5"
      style={{ gap: "0px" }}
    >
      {bins.map((bin, i) => (
        <div
          key={bin.timestamp}
          style={{
            width: 4,
            height: "100%",
            display: "flex",
            justifyContent: "center",
          }}
        >
          {i % step === 0 ? (
            <div
              style={{
                transform: "translateX(-50%)",
                whiteSpace: "nowrap",
              }}
            >
              {format(new Date(bin.timestamp), "HH:mm")}
            </div>
          ) : null}
        </div>
      ))}
    </div>
  );
};

export const TimelineView: React.FC<TimelineViewProps> = ({
  data: initialData,
  stream,
}) => {
  const [intervalIndex, setIntervalIndex] = useState(2); // Default 1hr
  const [liveData, setLiveData] = useState<TimelineBin[]>(initialData);
  const [from, setFrom] = useState(() => Date.now() - 24 * 60 * 60 * 1000); // Default last 24hrs
  const [to, setTo] = useState(() => Date.now());
  const interval = TIME_INTERVALS[intervalIndex];

  useEffect(() => {
    if (!stream) return;

    let active = true;
    const process = async () => {
      for await (const item of stream) {
        if (!active) break;
        setLiveData((prev) => [...prev, item]);
      }
    };

    process();
    return () => {
      active = false;
    };
  }, [stream]);

  const fullBinned = useMemo(
    () =>
      binData(
        liveData,
        interval,
        Number.MIN_SAFE_INTEGER,
        Number.MAX_SAFE_INTEGER,
      ),
    [liveData, interval],
  );

  const visibleBinned = useMemo(
    () =>
      fullBinned.filter((bin) => bin.timestamp >= from && bin.timestamp <= to),
    [fullBinned, from, to],
  );

  const handleWheel = useCallback(
    (e: WheelEvent) => {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        setIntervalIndex((prev) => {
          const delta = Math.sign(e.deltaY);
          const next = Math.min(
            Math.max(prev + delta, 0),
            TIME_INTERVALS.length - 1,
          );
          return next;
        });
      } else {
        const delta = Math.sign(e.deltaY) * interval;
        setFrom((prev) => prev + delta);
        setTo((prev) => prev + delta);
      }
    },
    [interval],
  );

  useEffect(() => {
    window.addEventListener("wheel", handleWheel, { passive: false });
    return () => {
      window.removeEventListener("wheel", handleWheel);
    };
  }, [handleWheel]);

  const handleZoomChange = useCallback(
    (e: React.ChangeEvent<HTMLSelectElement>) => {
      setIntervalIndex(Number(e.target.value));
    },
    [],
  );

  const rows = useMemo(
    () => [
      { label: "AFK", color: "gray", data: visibleBinned },
      { label: "VSCode", color: "purple", data: visibleBinned },
      { label: "General", color: "blue", data: visibleBinned },
    ],
    [visibleBinned],
  );

  const parentRef = useRef<HTMLDivElement | null>(null);
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 2,
  });

  return (
    <div className="p-4 space-y-2 overflow-x-auto">
      <div className="flex items-center gap-4 mb-2">
        <div className="flex items-center gap-2">
          <label className="text-sm text-gray-600">Time interval:</label>
          <select
            value={intervalIndex}
            onChange={handleZoomChange}
            className="border rounded px-2 py-1 text-sm"
          >
            {TIME_INTERVALS.map((ms, idx) => (
              <option key={idx} value={idx}>
                {ms / 1000 / 60 >= 60
                  ? `${ms / 1000 / 60 / 60}hr`
                  : `${ms / 1000 / 60}min`}
              </option>
            ))}
          </select>
        </div>

        <div className="flex items-center gap-2">
          <label className="text-sm text-gray-600">From:</label>
          <input
            type="datetime-local"
            className="border rounded px-2 py-1 text-sm"
            value={new Date(to).toISOString().slice(0, 16)}
            onChange={(e) => setFrom(new Date(e.target.value).getTime())}
          />
        </div>

        <div className="flex items-center gap-2">
          <label className="text-sm text-gray-600">To:</label>
          <input
            type="datetime-local"
            className="border rounded px-2 py-1 text-sm"
            value={new Date(to).toISOString().slice(0, 16)}
            onChange={(e) => setTo(new Date(e.target.value).getTime())}
          />
        </div>
      </div>

      <div ref={parentRef} className="overflow-y-auto h-[96px]">
        <div
          style={{
            height: virtualizer.getTotalSize(),
            position: "relative",
            width: Math.max(600, visibleBinned.length * 4 + LABEL_WIDTH),
          }}
        >
          {virtualizer.getVirtualItems().map((virtualRow) => {
            const row = rows[virtualRow.index];
            return (
              <div
                key={row.label}
                style={{
                  position: "absolute",
                  top: 0,
                  left: 0,
                  width: "100%",
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                <TrackRow
                  index={virtualRow.index}
                  style={{ height: ROW_HEIGHT }}
                  label={row.label}
                  color={row.color}
                  data={row.data}
                />
              </div>
            );
          })}
        </div>
      </div>

      <TimeAxis bins={visibleBinned} />
    </div>
  );
};
