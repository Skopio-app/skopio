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

export type RPCMessage = RequestMessage | ResponseMessage;
