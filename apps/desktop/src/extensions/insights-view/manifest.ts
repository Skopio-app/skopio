import {
  BaseBuiltInExtensionManifest,
  ExtensionTypeEnum,
} from "@skopio/manifest";
import { NotebookText } from "lucide-react";

export const manifest: BaseBuiltInExtensionManifest = {
  id: "builtin-5-manifest",
  name: "Insights",
  description: "A view of average usage stats grouped by year",
  type: ExtensionTypeEnum.Tab,
  icon: NotebookText,
  permissions: [],
};
