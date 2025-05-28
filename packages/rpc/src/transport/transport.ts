import { RPCMessage } from "../core/message-types";

export interface Transport {
  /**
   * Send a message to the connected endpoint
   */
  send(message: RPCMessage): void;
  /**
   * Register a callback to be called when a message is received
   */
  onMessage(
    callback: (message: RPCMessage, eventOrigin?: string) => void,
  ): void;
  /**
   * Dispose/close the transport and remove all listeners
   */
  dispose(): void;
}
