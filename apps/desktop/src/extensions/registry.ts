import {
  BaseBuiltInExtensionManifest,
  BaseBuiltInExtensionManifestSchema,
  ExtensionTypeEnum,
} from "@skopio/manifest";
import { ComponentType } from "react";

export interface ExtensionRegistration {
  manifest: BaseBuiltInExtensionManifest;
  component: ComponentType;
}

class BuiltinExtensionRegistry {
  private tabExtensions = new Map<string, ExtensionRegistration>();
  private widgetExtensions = new Map<string, ExtensionRegistration>();

  register(extension: ExtensionRegistration) {
    const { manifest } = extension;

    BaseBuiltInExtensionManifestSchema.parse(manifest);

    switch (manifest.type) {
      case ExtensionTypeEnum.Tab:
        this.tabExtensions.set(manifest.id, extension);
        break;
      case ExtensionTypeEnum.Widget:
        this.widgetExtensions.set(manifest.id, extension);
        break;
      default:
        throw new Error(`Unsupported extension type: ${manifest.type}`);
    }
  }

  getTabExtensions(): ExtensionRegistration[] {
    return Array.from(this.tabExtensions.values());
  }

  getWidgetExtensions(): ExtensionRegistration[] {
    return Array.from(this.widgetExtensions.values());
  }

  getExtensionById(id: string): ExtensionRegistration | undefined {
    return this.tabExtensions.get(id) || this.widgetExtensions.get(id);
  }

  clear() {
    this.tabExtensions.clear();
    this.widgetExtensions.clear();
  }
}

export const builtinExtensionRegistry = new BuiltinExtensionRegistry();
