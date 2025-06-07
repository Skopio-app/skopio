import {
  BaseBuiltInExtensionManifest,
  ExtensionTypeEnum,
} from "@skopio/manifest";

export const manifest: BaseBuiltInExtensionManifest = {
  id: "builtin-timeline",
  name: "Timeline",
  description:
    "A timeline view of the various captured events and their sources",
  type: ExtensionTypeEnum.Tab,
  icon: "",
  permissions: [],
};
