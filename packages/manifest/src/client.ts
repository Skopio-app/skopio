import z from "zod";

export type ToastParams = {
  description?: string;
  duration?: number;
  closeButton?: boolean;
  position?:
    | "top-left"
    | "top-right"
    | "bottom-right"
    | "bottom-left"
    | "top-center"
    | "bottom-center";
};

export type GeneralToast = (
  message: string,
  options?: ToastParams,
  action?: () => void,
) => Promise<void>;

export interface IToast {
  message: GeneralToast;
  info: GeneralToast;
  success: GeneralToast;
  warning: GeneralToast;
  error: GeneralToast;
}

export interface IOpen {
  url: (url: string) => Promise<void>;
  file: (path: string) => Promise<void>;
  folder: (path: string) => Promise<void>;
}

export type DragDropPayload = {
  paths: string[];
  position: { x: number; y: number };
};
export type DragEnterPayload = DragDropPayload;
export type DragOverPayload = {
  position: { x: number; y: number };
};

export interface IEvent {
  /**
   * Get files dropped on the window
   */
  onDragDrop: (callback: (payload: DragDropPayload) => void) => void;
  /**
   * Listen to drag event event, when mouse drag enters the window
   */
  onDragEnter: (callback: (payload: DragEnterPayload) => void) => void;
  /**
   * Listen to drag leave events, when mouse drag leaves the window
   */
  onDragLeave: (callback: () => void) => void;
  /**
   * Get the position of the dragged item
   */
  onDragOver: (callback: (payload: DragOverPayload) => void) => void;
  /**
   * Listen to window blur (defocus) event
   */
  onWindowBlur: (callback: () => void) => void;
  /**
   * Listen to window close request event
   */
  onWindowCloseRequested: (callback: () => void) => void;
  /**
   * Listen to window on focus event
   */
  onWindowFocus: (callback: () => void) => void;
}

export const MacSecurityOptions = z.union([
  z.literal("ScreenCapture"),
  z.literal("Camera"),
  z.literal("Microphone"),
  z.literal("Accessibility"),
  z.literal("AllFiles"),
]);
export type MacSecurityOptions = z.infer<typeof MacSecurityOptions>;

export interface ISecurity {
  mac: {
    revealSecurityPane: (privacyOption?: MacSecurityOptions) => Promise<void>;
    resetPermission: (privacyOption: MacSecurityOptions) => Promise<void>;
    verifyFingerPrint: () => Promise<boolean>;
    requestScreenCapturePermission: () => Promise<boolean>;
    checkScreenCapturePermission: () => Promise<boolean>;
  };
}
