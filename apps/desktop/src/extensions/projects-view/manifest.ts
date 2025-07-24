import {
  BaseBuiltInExtensionManifest,
  ExtensionTypeEnum,
} from "@skopio/manifest";
import { FolderKanban } from "lucide-react";

export const manifest: BaseBuiltInExtensionManifest = {
  id: "builtin-4-projects",
  name: "Projects",
  description:
    "A view of the various logged projects together with their stats and associated entities",
  type: ExtensionTypeEnum.Tab,
  icon: FolderKanban,
  permissions: [],
};
