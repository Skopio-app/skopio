import {
  BucketSummaryInput,
  commands,
  EventGroup,
  EventGroupResult,
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

export const useEventSummary = (
  group: Group,
  duration: number,
  customRange: CustomRange,
) => {
  const hasValidCustom =
    !!customRange && !!customRange.start && !!customRange.end;

  const { data, isPending, isFetching } = useQuery({
    queryKey: [
      "eventSummary",
      {
        group,
        duration: hasValidCustom ? undefined : duration,
        custom: hasValidCustom
          ? {
              start: customRange.start.toISOString(),
              end: customRange.end.toISOString(),
            }
          : null,
      },
    ],
    queryFn: async (): Promise<EventGroupResult> => {
      const input = buildInput(
        group,
        duration,
        hasValidCustom ? customRange : null,
      );
      return commands.fetchEvents(input);
    },
    enabled: hasValidCustom || !customRange,
    select: (res): EventGroup[] => ("Grouped" in res ? res.Grouped : []),
  });

  const loading = isPending || isFetching;

  return { events: data ?? [], loading };
};
