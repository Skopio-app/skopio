import {
  BaseBuiltInExtensionManifest,
  ExtensionTypeEnum,
} from "@skopio/manifest";
import { Flag } from "lucide-react";

export const manifest: BaseBuiltInExtensionManifest = {
  id: "builtin-3-goals",
  name: "Goals",
  description:
    "A view of the various set goals, with the ability to set, edit, or delete goals",
  type: ExtensionTypeEnum.Tab,
  icon: Flag,
  permissions: [],
};
