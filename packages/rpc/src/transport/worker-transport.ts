import { RPCMessage } from "../core/message-types";
import { Transport } from "./transport";

export class WorkerTransport implements Transport {
  private listener?: (event: MessageEvent) => void;

  constructor(private worker: Worker) {}

  send(message: RPCMessage): void {
    this.worker.postMessage(message);
  }

  onMessage(callback: (message: any, eventOrigin?: string) => void): void {
    this.listener = (event: MessageEvent) => {
      callback(event.data);
    };
    this.worker.addEventListener("message", this.listener);
  }

  dispose(): void {
    if (this.listener) {
      this.worker.removeEventListener("message", this.listener);
      this.listener = undefined;
    }
  }
}
