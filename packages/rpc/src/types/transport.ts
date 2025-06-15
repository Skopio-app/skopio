export interface IOInterface {
  name: string;
  read(): Promise<string | null>;
  write(data: string): Promise<void>;
}

export interface DestroyableIOInterface extends IOInterface {
  destroy(): void;
  signalDestroy(): void;
}
