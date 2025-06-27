// import { index, route, type RouteConfig } from "@react-router/dev/routes";

import { createBrowserRouter } from "react-router";
import App from "./App";
import TabExtensionPage from "./pages/TabExtensionPage";
import DashboardLayout from "./components/Dashboard";
import RedirectToTab from "./components/RedirectToTab";

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
            element: <RedirectToTab />,
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
