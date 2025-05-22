import { z } from "zod";

export const EventPermissionSchema = z.union([
  z.literal("event:drag-drop"),
  z.literal("event:drag-enter"),
  z.literal("event:drag-leave"),
  z.literal("event:drag-over"),
  z.literal("event:window-blur"),
  z.literal("event:window-close-requested"),
  z.literal("event:window-focus"),
]);
export type EventPermissionSchema = z.infer<typeof EventPermissionSchema>;

export const SecurityPermissionSchema = z.union([
  z.literal("security:mac:request-permission"),
  z.literal("security:mac:check-permission"),
  z.literal("security:mac:all"),
]);
export type SecurityPermissionSchema = z.infer<typeof SecurityPermissionSchema>;
