import {
  BaseBuiltInExtensionManifest,
  ExtensionTypeEnum,
} from "@skopio/manifest";
import { ChartNoAxesGantt } from "lucide-react";

export const manifest: BaseBuiltInExtensionManifest = {
  id: "builtin-2-timeline",
  name: "Timeline",
  description:
    "A timeline view of the various captured events and their sources",
  type: ExtensionTypeEnum.Tab,
  icon: ChartNoAxesGantt,
  permissions: [],
};
