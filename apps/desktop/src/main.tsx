import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import { RouterProvider } from "react-router";
import routes from "./routes.tsx";
import { registerBuiltinExtensions } from "./extensions/registerBuiltinExtensions.ts";

registerBuiltinExtensions();
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <RouterProvider router={routes} />
  </StrictMode>,
);
