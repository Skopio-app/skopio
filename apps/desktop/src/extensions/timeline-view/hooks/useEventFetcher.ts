import { useEffect, useState } from "react";
import {
  BucketSummaryInput,
  commands,
  EventGroup,
  EventGroupResult,
  Group,
} from "../../../types/tauri.gen";

export const useEventFetcher = (
  group: Group,
  duration: number,
  customRange: { start: Date; end: Date } | null,
) => {
  const [events, setEvents] = useState<EventGroup[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (customRange && (!customRange.start || !customRange.end)) return;

    const fetch = async () => {
      const input: BucketSummaryInput = customRange
        ? {
            preset: {
              custom: {
                start: customRange.start.toISOString(),
                end: customRange.end.toISOString(),
                bucket: "day",
              },
            },
            groupBy: group,
          }
        : {
            preset: { lastNMinutes: duration },
            groupBy: group,
          };

      try {
        const result: EventGroupResult = await commands.fetchEvents(input);
        if ("Grouped" in result) {
          setEvents(result.Grouped);
        }
      } catch (err) {
        console.error("Failed to fetch events: ", err);
      } finally {
        setLoading(false);
      }
    };

    fetch();
  }, [group, duration, customRange]);

  return { events, loading };
};
