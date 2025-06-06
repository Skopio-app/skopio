// import { index, route, type RouteConfig } from "@react-router/dev/routes";

import { createBrowserRouter } from "react-router";
import App from "./App";
import { ExtensionIfrane } from "./extension-iframe";
import DashboardPage from "./pages/DashboardPage";

// export default [
//     route("dashboard", "./App.tsx", [
//         index("./home.tsx")
//     ]),
// ] satisfies RouteConfig

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    children: [
      {
        index: true,
        element: <DashboardPage />,
      },
      {
        path: "app/extension",
        element: <ExtensionIfrane />,
      },
    ],
  },
]);

export default router;
