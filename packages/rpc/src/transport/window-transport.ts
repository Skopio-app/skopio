import { RPCMessage } from "../core/message-types";
import { Transport } from "./transport";

export class WindowTransport implements Transport {
  private listener?: (event: MessageEvent) => void;

  constructor(
    private targetWindow: Window,
    private targetOrigin: string = "*",
  ) {}
  send(message: RPCMessage): void {
    this.targetWindow.postMessage(message, this.targetOrigin);
  }
  onMessage(callback: (message: any, eventOrigin?: string) => void): void {
    this.listener = (event: MessageEvent) => {
      if (event.source === this.targetWindow) {
        callback(event.data, event.origin);
      }
    };
    window.addEventListener("message", this.listener);
  }
  dispose(): void {
    if (this.listener) {
      window.removeEventListener("message", this.listener);
    }
  }
}
