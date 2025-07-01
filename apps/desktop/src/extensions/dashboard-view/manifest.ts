import {
  BaseBuiltInExtensionManifest,
  ExtensionTypeEnum,
} from "@skopio/manifest";
import { LayoutDashboard } from "lucide-react";

export const manifest: BaseBuiltInExtensionManifest = {
  id: "builtin-1-dashboard",
  name: "Dashboard",
  description:
    "A dashboard view of the the various captured stats and their durations.",
  type: ExtensionTypeEnum.Tab,
  icon: LayoutDashboard,
  permissions: [],
};
