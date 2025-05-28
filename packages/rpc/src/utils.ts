import {
  RequestMessage,
  ResponseMessage,
  RPCMessage,
} from "./core/message-types";

/**
 * Generates a unique request ID
 */
export const generateId = (): string => {
  return `rpc-${Date.now()}-${Math.random().toString(36).slice(2)}`;
};

/**
 * Type guard for RequestMessage
 */
export const isRequestMessage = (msg: RPCMessage): msg is RequestMessage => {
  return msg && msg.type === "request" && "method" in msg;
};

/**
 * Type guard for ResponseMessage
 */
export const isResponseMessage = (msg: RPCMessage): msg is ResponseMessage => {
  return msg && msg.type === "response" && "id" in msg;
};

/**
 * Safe invoke: runs a function and returns [result, error]
 */
export const tryInvoke = async <T>(
  fn: () => Promise<T> | T,
): Promise<[T | null, Error | null]> => {
  try {
    const result = await fn();
    return [result, null];
  } catch (err) {
    return [null, err instanceof Error ? err : new Error(String(err))];
  }
};
