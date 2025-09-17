export const LAST_ACTIVE_TAB = "lastActiveId";

export const AFK = [
  ["30s", "30 seconds", 30],
  ["1m", "1 minute", 60],
  ["2m", "2 minutes", 120],
  ["3m", "3 minutes", 180],
  ["5m", "5 minutes", 300],
  ["10m", "10 minutes", 600],
] as const;

export type AfkKey = (typeof AFK)[number][0];
export type AfkSeconds = (typeof AFK)[number][2];

export const AFK_KEYS = AFK.map(([key]) => key) as readonly AfkKey[];
export const AFK_SECONDS = Object.fromEntries(
  AFK.map(([k, , s]) => [k, s]),
) as Record<AfkKey, AfkSeconds>;
export const AFK_LABELS = Object.fromEntries(
  AFK.map(([k, , s]) => [s, k]),
) as Record<number, AfkKey>;
