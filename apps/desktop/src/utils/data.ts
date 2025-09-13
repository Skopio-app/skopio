import { PermissionStatus } from "@/types/tauri.gen";

export const getEntityName = (
  fullPath: string,
  metadata: string | null,
): string => {
  if (metadata !== null && metadata === "File") {
    return fullPath.split(/[\\/]/).pop() || fullPath;
  }
  return fullPath;
};

export const isGranted = (s: PermissionStatus) => s === "granted";

export const truncateValue = (text: string, limit?: number): string => {
  const effectiveLimit = limit ?? 50;
  return text.length > effectiveLimit
    ? `${text.slice(0, effectiveLimit)}...`
    : text;
};
