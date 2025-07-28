/**
 * Formats a duration given in seconds into a human-readable string.
 * @param seconds - The duration in seconds
 * @returns A string like "1h 45m 30s"
 */
export const formatDuration = (seconds: number): string => {
  const roundedSecs = Math.round(seconds);
  const hrs = Math.floor(roundedSecs / 3600);
  const mins = Math.floor((roundedSecs % 3600) / 60);
  const secs = roundedSecs % 60;

  const padded = (n: number) => String(n).padStart(2, "0");
  const hrStr = `${hrs}h`;
  const minStr = `${padded(mins)}m`;
  const secStr = `${padded(secs)}s`;
  if (hrs > 0) {
    return `${hrStr} ${minStr} ${secStr}`;
  } else if (mins > 0) {
    return `${minStr} ${secStr}`;
  } else {
    return `${secStr}`;
  }
};
