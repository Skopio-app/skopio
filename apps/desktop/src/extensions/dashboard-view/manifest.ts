import {
  BaseBuiltInExtensionManifest,
  ExtensionTypeEnum,
} from "@skopio/manifest";

export const manifest: BaseBuiltInExtensionManifest = {
  id: "builtin-dashboard",
  name: "Dashboard",
  description:
    "A dashboard view of the the various captured stats and their durations.",
  type: ExtensionTypeEnum.Tab,
  icon: "",
  permissions: [],
};
