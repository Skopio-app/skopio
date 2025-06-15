export const generateId = (): string => {
  return `rpc-${Date.now()}-${Math.random().toString(36).slice(2)}`;
};
