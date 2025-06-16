// import { index, route, type RouteConfig } from "@react-router/dev/routes";

import { createBrowserRouter } from "react-router";
import App from "./App";
import DashboardPage from "./pages/DashboardPage";
import TabExtensionPage from "./pages/TabExtensionPage";
import DashboardLayout from "./components/Dashboard";

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
        element: <DashboardLayout />,
        children: [
          {
            index: true,
            element: <DashboardPage />,
          },
          {
            path: "tab/:id",
            element: <TabExtensionPage />,
          },
        ],
      },
    ],
  },
]);

export default router;
