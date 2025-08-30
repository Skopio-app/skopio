import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import { RouterProvider } from "react-router";
import routes from "@/routes.tsx";
import { registerBuiltinExtensions } from "@/extensions/registerBuiltinExtensions.ts";

document.addEventListener("DOMContentLoaded", () => {
  const dragRegionDiv = document.createElement("div");
  dragRegionDiv.setAttribute("data-tauri-drag-region", "");
  dragRegionDiv.className = "draggable-state";
  document.documentElement.insertBefore(dragRegionDiv, document.body);
});

registerBuiltinExtensions();
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={routes} />
  </StrictMode>,
);
