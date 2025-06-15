import z from "zod";

export const PermissionScopeSchema = z.object({
  path: z.optional(z.string()),
  url: z.optional(z.string()),
  cmd: z.optional(
    z.object({
      program: z.string(),
      args: z.array(z.string()),
    }),
  ),
});

export const EventPermissionSchema = z.union([
  z.literal("event:drag-drop"),
  z.literal("event:drag-enter"),
  z.literal("event:drag-leave"),
  z.literal("event:drag-over"),
  z.literal("event:window-blur"),
  z.literal("event:window-close-requested"),
  z.literal("event:window-focus"),
]);
export type EventPermission = z.infer<typeof EventPermissionSchema>;

export const SecurityPermissionSchema = z.union([
  z.literal("security:mac:reveal-security-pane"),
  z.literal("security:mac:verify-fingerprint"),
  z.literal("security:mac:reset-screencapture-permission"),
  z.literal("security:mac:request-permission"),
  z.literal("security:mac:check-permission"),
  z.literal("security:mac:all"),
]);
export type SecurityPermission = z.infer<typeof SecurityPermissionSchema>;

export const OpenPermissionSchema = z.union([
  z.literal("open:url"),
  z.literal("open:file"),
  z.literal("open:folder"),
]);

export const OpenPermissionScopedSchema = z.object({
  permission: OpenPermissionSchema,
  allow: z.optional(z.array(PermissionScopeSchema)),
  deny: z.optional(z.array(PermissionScopeSchema)),
});
export type OpenPermissionScoped = z.infer<typeof OpenPermissionScopedSchema>;

export const SkopioManifestPermission = z.union([
  EventPermissionSchema,
  SecurityPermissionSchema,
]);

export const AllSkopioPermission = z.union([
  SkopioManifestPermission,
  OpenPermissionSchema,
]);
export type AllSkopioPermission = z.infer<typeof AllSkopioPermission>;
