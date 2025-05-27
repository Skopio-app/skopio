import z from "zod";

/**
 * Map window label to extension
 */
export const ExtensionLabelMap = z.record(
  z.string(), // Window label
  z.object({
    path: z.string().describe("Path to the extension"),
    processes: z.number().array(),
    dist: z.string().optional().nullable(),
  }),
);
export type ExtensionLabelMap = z.infer<typeof ExtensionLabelMap>;

export const Extension = z.object({
  extId: z.number(),
  identifier: z.string(),
  version: z.string(),
  enabled: z.boolean(),
  installedAt: z.string(),
  path: z.string().optional().nullable(),
  data: z.any().optional(),
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
  alias: z.string().optional().nullable(),
  hotkey: z.string().optional().nullable(),
  enabled: z.boolean(),
});
export type ExtensionCommand = z.infer<typeof ExtensionCommand>;

export const ExtensionData = z.object({
  dataId: z.number(),
  extId: z.number(),
  dataType: z.string(),
  data: z.string().optional(),
  searchText: z.string().optional(),
  createdAt: z.date(),
  updatedAt: z.date(),
});
export type ExtensionData = z.infer<typeof ExtensionData>;
