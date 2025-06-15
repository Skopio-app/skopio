import { BaseBuiltInExtensionManifest } from "@skopio/manifest";
import { ComponentType } from "react";
import { builtinExtensionRegistry } from "./registry";

// Load all 'manifest.ts' files under builtin extensions
const manifestModules = import.meta.glob("./*/manifest.ts", {
  eager: true,
}) as Record<string, { manifest: BaseBuiltInExtensionManifest }>;

// Load all 'index.tsx' files under builtin extensions
const componentModules = import.meta.glob("./*/index.tsx", {
  eager: true,
}) as Record<string, { default: ComponentType }>;

export const registerBuiltinExtensions = () => {
  for (const path in manifestModules) {
    const basePath = path.replace("/manifest.ts", "");
    const manifest = manifestModules[path].manifest;
    const componentPath = `${basePath}/index.tsx`;
    const component = componentModules[componentPath]?.default;

    if (!component) {
      console.warn(`No component found for ${manifest.id} at ${componentPath}`);
      continue;
    }

    builtinExtensionRegistry.register({
      manifest,
      component,
    });
  }
};
