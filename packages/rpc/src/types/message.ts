export type RequestMessage = {
  id: string;
  type: "request";
  method: string;
  payload: unknown;
};

export type ResponseMessage = {
  id: string;
  type: "response";
  result?: unknown;
  error?: string;
};

export type CallbackMessage = {
  id: string;
  type: "callback";
  method: string;
  args: unknown;
};

export type StreamMessage = {
  id: string;
  type: "stream";
  method: string;
  args: unknown[];
};

export type RPCMessage =
  | RequestMessage
  | ResponseMessage
  | CallbackMessage
  | StreamMessage;
