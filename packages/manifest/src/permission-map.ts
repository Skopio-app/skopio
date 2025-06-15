import { IEvent, ISecurity } from "./client";
import { EventPermission, SecurityPermission } from "./schema";

export const EventPermissionMap: Record<keyof IEvent, EventPermission[]> = {
  onDragDrop: ["event:drag-drop"],
  onDragEnter: ["event:drag-enter"],
  onDragLeave: ["event:drag-leave"],
  onDragOver: ["event:drag-over"],
  onWindowBlur: ["event:window-blur"],
  onWindowCloseRequested: ["event:window-close-requested"],
  onWindowFocus: ["event:window-focus"],
};

export const SecurityPermissionMap: {
  mac: Record<keyof ISecurity["mac"], SecurityPermission[]>;
} = {
  mac: {
    revealSecurityPane: [
      "security:mac:all",
      "security:mac:reveal-security-pane",
    ],
    verifyFingerPrint: ["security:mac:all", "security:mac:verify-fingerprint"],
    requestScreenCapturePermission: [
      "security:mac:all",
      "security:mac:request-permission",
    ],
    checkScreenCapturePermission: [
      "security:mac:all",
      "security:mac:check-permission",
    ],
    resetPermission: [
      "security:mac:all",
      "security:mac:reset-screencapture-permission",
    ],
  },
};
