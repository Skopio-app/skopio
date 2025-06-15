import { DestroyableIOInterface } from "../types/transport";

const DESTROY_SIGNAL = "__DESTROY__";

export class WorkerIO implements DestroyableIOInterface {
  name = "worker-io";
  private resolveRead: ((data: string | null) => void) | null = null;
  private queue: string[] = [];

  constructor(private worker: Worker) {
    this.worker.onmessage = (e: MessageEvent) => this.enqueue(e.data);
  }

  private enqueue(data: string) {
    if (data === DESTROY_SIGNAL) {
      this.destroy();
      return;
    }

    if (this.resolveRead) {
      this.resolveRead(data);
      this.resolveRead = null;
    } else {
      this.queue.push(data);
    }
  }

  async read(): Promise<string | null> {
    if (this.queue.length > 0) {
      return this.queue.shift() ?? null;
    }

    return new Promise((resolve) => {
      this.resolveRead = resolve;
    });
  }

  async write(data: string): Promise<void> {
    this.worker.postMessage(data);
  }

  destroy(): void {
    this.worker.terminate();
    this.resolveRead = null;
    this.queue.length = 0;
  }

  signalDestroy(): void {
    this.worker.postMessage(DESTROY_SIGNAL);
  }
}
