import { AllSkopioPermission } from "./schema";

export type PermissionDescriptions = Record<AllSkopioPermission, string>;

export const permissionDescriptions: PermissionDescriptions = {
  /*----------------------------------------------------------*/
  /*                     Event                                */
  /*----------------------------------------------------------*/
  "event:drag-drop": "Listen to file drop event",
  "event:drag-enter": "Listen to drag enter event",
  "event:drag-leave": "Listen to drag leave event",
  "event:drag-over": "Listen to drag over event",
  "event:window-blur": "Listen to window blur event",
  "event:window-close-requested": "Listen to window close requested event",
  "event:window-focus": "Listen to window focus event",
  /*----------------------------------------------------------*/
  /*                     Security                             */
  /*----------------------------------------------------------*/
  "security:mac:request-permission": "Request security permission",
  "security:mac:check-permission": "Check security permission",
  "security:mac:all": "All Mac Security APIs",
  "security:mac:reveal-security-pane":
    "Reveal security privacy settings panel in Mac's System Preferences",
  "security:mac:verify-fingerprint": "Verify fingerprint",
  "security:mac:reset-screencapture-permission": "Reset permission",
  /*----------------------------------------------------------*/
  /*                     Open                                 */
  /*----------------------------------------------------------*/
  "open:url": "Open URLs",
  "open:file": "Open files",
  "open:folder": "Open folders",
};
