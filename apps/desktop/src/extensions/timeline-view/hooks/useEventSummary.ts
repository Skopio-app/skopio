import {
  BucketSummaryInput,
  commands,
  EventGroup,
  EventGroupResult,
  FullEvent,
  Group,
} from "@/types/tauri.gen";
import { useQuery } from "@tanstack/react-query";

type CustomRange = { start: Date; end: Date } | null;

const buildInput = (
  group: Group,
  duration: number,
  customRange: CustomRange,
): BucketSummaryInput => {
  if (customRange) {
    return {
      preset: {
        custom: {
          start: customRange.start.toISOString(),
          end: customRange.end.toISOString(),
          bucket: "day",
        },
      },
      groupBy: group,
    };
  }
  return {
    preset: { lastNMinutes: duration },
    groupBy: group,
  };
};

const toGrouped = (res: EventGroupResult): EventGroup[] =>
  "Grouped" in res ? res.Grouped : [];

export const useEventSummary = (
  group: Group,
  duration: number,
  customRange: CustomRange,
  showAfk: boolean,
) => {
  const hasValidCustom =
    !!customRange && !!customRange.start && !!customRange.end;

  const queryKeyBase = {
    group,
    duration: hasValidCustom ? undefined : duration,
    custom: hasValidCustom
      ? {
          start: customRange.start.toISOString(),
          end: customRange.end.toISOString(),
        }
      : null,
  };

  const input = buildInput(
    group,
    duration,
    hasValidCustom ? customRange : null,
  );

  const { data: eventRes, isLoading: eventsLoading } = useQuery({
    queryKey: ["eventSummary", queryKeyBase],
    queryFn: async (): Promise<EventGroupResult> => {
      return commands.fetchEvents(input);
    },
    enabled: hasValidCustom || !customRange,
    select: toGrouped,
  });

  const { data: afkRes, isLoading: afkLoading } = useQuery({
    queryKey: ["afkEvents", queryKeyBase],
    queryFn: async (): Promise<FullEvent[]> => {
      return commands.fetchAfkEvents(input);
    },
    enabled: (hasValidCustom || !customRange) && showAfk,
  });

  return {
    events: eventRes ?? [],
    afkEvents: afkRes ?? [],
    loading: eventsLoading || (showAfk ? afkLoading : false),
  };
};
