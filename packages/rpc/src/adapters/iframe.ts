import { DestroyableIOInterface } from "../types/transport";

const DESTROY_SIGNAL = "__DESTROY__";
const PORT_INIT_SIGNAL = "__PORT_INIT__";

export class IframeIO implements DestroyableIOInterface {
  name = "iframe-io";
  private port: MessagePort | null = null;
  private queue: string[] = [];
  private resolveRead: ((value: string | null) => void) | null = null;

  constructor(private targetWindow: Window) {
    window.addEventListener("message", this.handleInit);
  }

  private handleInit = (event: MessageEvent) => {
    if (event.source !== this.targetWindow) return;

    if (event.data === PORT_INIT_SIGNAL && event.ports.length > 0) {
      this.port = event.ports[0];
      this.port.onmessage = this.handleMessage;

      // Flush queue
      while (this.queue.length > 0) {
        const msg = this.queue.shift();
        if (msg) this.port.postMessage(msg);
      }
    }
  };

  private handleMessage = (event: MessageEvent) => {
    const message = event.data;

    if (message === DESTROY_SIGNAL) {
      this.destroy();
      return;
    }

    if (this.resolveRead) {
      this.resolveRead(message);
      this.resolveRead = null;
    } else {
      this.queue.push(message);
    }
  };

  async read(): Promise<string | null> {
    if (this.queue.length > 0) {
      return this.queue.shift() ?? null;
    }

    return new Promise((resolve) => {
      this.resolveRead = resolve;
    });
  }

  async write(data: string): Promise<void> {
    if (!this.port) {
      this.queue.push(data);
      return;
    }

    this.port.postMessage(data);
  }

  destroy(): void {
    if (this.port) {
      this.port.close();
      this.port = null;
    }
    this.resolveRead = null;
    this.queue.length = 0;
  }

  signalDestroy(): void {
    if (this.port) {
      this.port.postMessage(DESTROY_SIGNAL);
    }
  }
}
