import z from "zod";
import { ExtensionType, ExtensionTypeEnum } from "./extension";
import {
  OpenPermissionScopedSchema,
  SkopioManifestPermission,
} from "../schema";
import { FC } from "react";

export const WidgetConfig = z.object({
  width: z.number().optional().nullable(),
  height: z.number().optional().nullable(),
  minWidth: z.number().optional().nullable(),
  minHeight: z.number().optional().nullable(),
  maxWidth: z.number().optional().nullable(),
  maxHeight: z.number().optional().nullable(),
  resizable: z.number().optional().nullable(),
  title: z.string().optional().nullable(),
});
export type WidgetConfig = z.infer<typeof WidgetConfig>;

export const BaseExtension = z.object({
  main: z.string().describe("The HTML files to load"),
  description: z.string().optional().nullable(),
  name: z.string(),
  dist: z.string().describe("Dist folder to load"),
  devUrl: z.string().describe("The URL to load in development"),
});

export const BaseBuiltInExtensionManifestSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string(),
  type: ExtensionType,
  icon: z.custom<FC>(),
  permissions: z.array(z.string()),
});
export type BaseBuiltInExtensionManifest = z.infer<
  typeof BaseBuiltInExtensionManifestSchema
>;

export const TabExtension = BaseExtension.extend({
  type: ExtensionType.optional().default(ExtensionTypeEnum.Tab),
});
export type TabExtension = z.infer<typeof TabExtension>;

export const WidgetExtension = BaseExtension.extend({
  type: ExtensionType.optional().default(ExtensionTypeEnum.Widget),
});
export type WidgetExtension = z.infer<typeof WidgetExtension>;

export const PermissionUnion = z.union([
  SkopioManifestPermission,
  OpenPermissionScopedSchema,
]);
export type PermissionUnion = z.infer<typeof PermissionUnion>;

export const SkopioExtensionManifest = z.object({
  name: z.string(),
  shortDescription: z.string(),
  longDescription: z.string(),
  identifier: z.string().describe("The unique identifier for the extension"),
  permissions: z
    .array(PermissionUnion)
    .describe(
      "Permissions declared by the extensions. Not declared APIs will be blocked.",
    ),
  tabExtension: z.array(TabExtension).optional(),
  widgetExtension: z.array(WidgetExtension).optional(),
});
export type SkopioExtensionManifest = z.infer<typeof SkopioExtensionManifest>;

export const ExtensionPackageJson = z.object({
  name: z.string(),
  version: z.string(),
  license: z.string(),
  author: z.string().optional(),
  repository: z.optional(
    z.union([
      z.string().describe("The URL of the repository"),
      z.object({
        type: z.string().describe("The type of repository"),
        url: z.string(),
      }),
    ]),
  ),
  dependencies: z.record(z.string(), z.string()).optional(),
  skopio: SkopioExtensionManifest,
  files: z
    .array(z.string())
    .optional()
    .describe("Files to include in the extension"),
});
export type ExtensionPackageJson = z.infer<typeof ExtensionPackageJson>;
