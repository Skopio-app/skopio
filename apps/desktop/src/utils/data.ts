export const getEntityName = (
  fullPath: string,
  metadata: string | null,
): string => {
  if (metadata !== null && metadata === "File") {
    return fullPath.split(/[\\/]/).pop() || fullPath;
  }
  return fullPath;
};
