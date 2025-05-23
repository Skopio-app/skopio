import z from "zod";

/**
 * Map window label to extension
 */
export const ExtensionLabelMap = z.record(
  z.string(), // Window label
  z.object({
    path: z.string(), // Path to the extension
    processes: z.array(z.number()),
    dist: z.optional(z.nullable(z.string())),
  }),
);
export type ExtensionLabelMap = z.infer<typeof ExtensionLabelMap>;

export const Extension = z.object({
  extId: z.number(),
  identifier: z.string(),
  version: z.string(),
  enabled: z.boolean(),
  installedAt: z.string(),
  path: z.optional(z.nullable(z.string())),
  data: z.optional(z.any()),
});
export type Extension = z.infer<typeof Extension>;

export enum ExtensionTypeEnum {
  Tab = "tab",
  Widget = "widget",
}
export const ExtensionType = z.nativeEnum(ExtensionTypeEnum);
export type ExtensionType = z.infer<typeof ExtensionType>;

export const ExtensionCommand = z.object({
  cmdId: z.number(),
  extId: z.number(),
  name: z.string(),
  type: ExtensionType,
  data: z.string(),
  alias: z.optional(z.nullable(z.string())),
  hotkey: z.optional(z.nullable(z.string())),
  enabled: z.boolean(),
});
export type ExtensionCommand = z.infer<typeof ExtensionCommand>;

export const ExtensionData = z.object({
  dataId: z.number(),
  extId: z.number(),
  dataType: z.string(),
  data: z.optional(z.string()),
  searchText: z.optional(z.string()),
  createdAt: z.date(),
  updatedAt: z.date(),
});
export type ExtensionData = z.infer<typeof ExtensionData>;
