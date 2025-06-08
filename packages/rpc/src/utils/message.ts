import {
  CallbackMessage,
  RequestMessage,
  ResponseMessage,
  RPCMessage,
  StreamMessage,
} from "../types/message";

export const isRequestMessage = (msg: RPCMessage): msg is RequestMessage => {
  return msg.type === "request" && "method" in msg;
};

export const isResponseMessage = (msg: RPCMessage): msg is ResponseMessage => {
  return msg.type === "response" && "id" in msg;
};

export const isCallbackMessage = (msg: RPCMessage): msg is CallbackMessage => {
  return msg.type === "callback" && "method" in msg;
};

export const isStreamMessage = (msg: RPCMessage): msg is StreamMessage => {
  return msg.type === "stream" && "method" in msg;
};
