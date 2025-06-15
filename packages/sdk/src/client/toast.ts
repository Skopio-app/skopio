import { rpc } from "@skopio/rpc";
import { toastSchema } from "../types/toast";

type ToastMethod =
  | "toast.success"
  | "toast.error"
  | "toast.info"
  | "toast.warning"
  | "toast.message";

async function sendToast(
  method: ToastMethod,
  message: string,
  options?: GeneralToastParams,
  action?: () => void,
): Promise<void> {
  const payload = {
    ...options,
    message,
  };

  // Validate before sending
  toastSchema.parse(payload);

  await rpc.invoke(method, payload);

  if (options?.actionLabel && action) {
    action(); // optional local callback
  }
}

export const toast = {
  message: (msg: string, opts?: GeneralToastParams, action?: () => void) =>
    sendToast("toast.message", msg, opts, action),

  success: (msg: string, opts?: GeneralToastParams, action?: () => void) =>
    sendToast("toast.success", msg, opts, action),

  error: (msg: string, opts?: GeneralToastParams, action?: () => void) =>
    sendToast("toast.error", msg, opts, action),

  info: (msg: string, opts?: GeneralToastParams, action?: () => void) =>
    sendToast("toast.info", msg, opts, action),

  warning: (msg: string, opts?: GeneralToastParams, action?: () => void) =>
    sendToast("toast.warning", msg, opts, action),
};
