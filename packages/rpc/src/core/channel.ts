import { ZodTypeAny } from "zod";
import { IOInterface } from "../types/transport";
import { generateId } from "../utils/id";
import {
  CallbackMessage,
  RequestMessage,
  ResponseMessage,
  RPCMessage,
  StreamMessage,
} from "../types/message";
import {
  isCallbackMessage,
  isRequestMessage,
  isResponseMessage,
  isStreamMessage,
} from "../utils/message";

export interface RPCMethod {
  input: ZodTypeAny;
  output: ZodTypeAny;
  handler: (input: any, context?: any) => any | Promise<any>;
  streamHandler?: (input: any, emit: (data: any) => void) => void;
}

interface RPCSchema {
  [method: string]: RPCMethod;
}

export class RPCChannel {
  private exposed: RPCSchema = {};
  private pending = new Map<
    string,
    { resolve: (val: any) => void; reject: (err: any) => void }
  >();
  private callbacks = new Map<string, (...args: any[]) => void>();
  private subscriptions = new Map<string, (data: any) => void>();

  constructor(
    private io: IOInterface,
    private debug = false,
  ) {
    this.listen();
  }

  expose(methods: RPCSchema) {
    Object.assign(this.exposed, methods);
  }

  async call<T = any>(
    method: string,
    input: unknown,
    timeout = 10000,
  ): Promise<T> {
    const id = generateId();
    const message: RequestMessage = {
      id,
      type: "request",
      method,
      payload: input,
    };

    const promise = new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id);
          reject(new Error(`RPC timeout: ${method}`));
        }
      }, timeout);
    });

    this.io.write(JSON.stringify(message));
    return promise;
  }

  subscribe<T = any>(
    method: string,
    input: unknown,
    onData: (data: T) => void,
  ): string {
    const id = generateId();
    this.subscriptions.set(id, onData);

    const message: StreamMessage = {
      id,
      type: "stream",
      method,
      args: [input],
    };
    this.io.write(JSON.stringify(message));
    return id;
  }

  unsubscribe(subscriptionId: string): void {
    this.subscriptions.delete(subscriptionId);
  }

  private async listen() {
    while (true) {
      const msgStr = await this.io.read();
      if (!msgStr) continue;

      let msg: RPCMessage;
      try {
        msg = JSON.parse(msgStr);
      } catch (error) {
        if (this.debug) console.warn("invalid JSON", msgStr, error);
        continue;
      }

      if (isRequestMessage(msg)) await this.handleRequest(msg);
      else if (isResponseMessage(msg)) this.handleResponse(msg);
      else if (isCallbackMessage(msg)) this.handleCallback(msg);
      else if (isStreamMessage(msg)) await this.handleStream(msg);
    }
  }

  private async handleRequest(msg: RequestMessage) {
    const method = this.exposed[msg.method];
    if (!method) return this.sendError(msg.id, `Unknown method: ${msg.method}`);

    try {
      const parsedInput = method.input.parse(msg.payload);
      const result = await method.handler(parsedInput);
      const parsedOutput = method.output.parse(result);
      const response: ResponseMessage = {
        id: msg.id,
        type: "response",
        result: parsedOutput,
      };
      this.io.write(JSON.stringify(response));
    } catch (err: any) {
      this.sendError(msg.id, err.message);
    }
  }

  private async handleStream(msg: StreamMessage) {
    const method = this.exposed[msg.method];
    if (!method?.streamHandler) return;
    const [input] = msg.args;
    const validatedInput = method.input.parse(input);
    method.streamHandler(validatedInput, (data) => {
      const callback: CallbackMessage = {
        id: msg.id,
        type: "callback",
        method: msg.id,
        args: [data],
      };
      this.io.write(JSON.stringify(callback));
    });
  }

  private handleResponse(msg: ResponseMessage) {
    const pending = this.pending.get(msg.id);
    if (!pending) return;
    this.pending.delete(msg.id);

    if (msg.error) pending.reject(new Error(msg.error));
    else pending.resolve(msg.result);
  }

  private handleCallback(msg: CallbackMessage) {
    const cb =
      this.callbacks.get(msg.method) || this.subscriptions.get(msg.method);
    if (cb && Array.isArray(msg.args)) {
      cb(...msg.args);
    } else {
      console.warn("Invalid callback args", msg.args);
    }
  }

  private sendError(id: string, error: string) {
    const response: ResponseMessage = { id, type: "response", error };
    this.io.write(JSON.stringify(response));
  }

  registerCallback(id: string, cb: (...args: any[]) => void) {
    this.callbacks.set(id, cb);
  }

  destroy() {
    this.pending.clear();
    this.callbacks.clear();
    this.subscriptions.clear();
    if ("destroy" in this.io) (this.io as any).destroy();
  }
}
