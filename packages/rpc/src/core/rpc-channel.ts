import { ZodTypeAny } from "zod";
import { Transport } from "../transport/transport";
import { RequestMessage, RPCMessage } from "./message-types";
import { generateId, isRequestMessage, isResponseMessage } from "../utils";

type RpcMethod = {
  input: ZodTypeAny;
  output: ZodTypeAny;
  handler: (input: any) => any | Promise<any>;
};

type RpcSchema = Record<string, RpcMethod>;

export class RPCChannel {
  private exposed: RpcSchema = {};
  private pending = new Map<
    string,
    {
      resolve: (val: any) => void;
      reject: (err: any) => void;
    }
  >();
  private counter = 0;

  constructor(private transport: Transport) {
    this.transport.onMessage(this.handleMessage.bind(this));
  }

  expose(methods: RpcSchema) {
    Object.assign(this.exposed, methods);
  }

  async call<T = any>(method: string, input: unknown): Promise<T> {
    const id = generateId();
    const msg: RequestMessage = {
      id,
      type: "request",
      method,
      payload: input,
    };

    const promise = new Promise<T>((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
    });

    this.transport.send(msg);
    return promise;
  }

  private async handleMessage(message: RPCMessage) {
    if (isRequestMessage(message)) {
      const method = this.exposed[message.method];
      if (!method) {
        this.transport.send({
          id: message.id,
          type: "response",
          error: `Unknown method: ${message.method}`,
        });
        return;
      }

      try {
        const validatedInput = method.input.parse(message.payload);
        const result = await method.handler(validatedInput);
        const validatedOutput = method.output.parse(result);
        this.transport.send({
          id: message.id,
          type: "response",
          result: validatedOutput,
        });
      } catch (err: any) {
        this.transport.send({
          id: message.id,
          type: "response",
          error: err.message || "Unknown error",
        });
      }
    } else if (isResponseMessage(message)) {
      const pending = this.pending.get(message.id);
      if (!pending) return;

      this.pending.delete(message.id);

      if (message.error) {
        pending.reject(new Error(message.error));
      } else {
        pending.resolve(message.result);
      }
    }
  }

  dispose() {
    this.pending.clear();
    this.transport.dispose();
  }
}
