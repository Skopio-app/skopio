import React from "react";
import * as ReactDOM from "react-dom/client";
import { NotificationPanel } from "./components/NotificationPanel";
import "./index.css";

ReactDOM.createRoot(
  document.getElementById("notification-root") as HTMLElement,
).render(
  <React.StrictMode>
    <NotificationPanel />
  </React.StrictMode>,
);
