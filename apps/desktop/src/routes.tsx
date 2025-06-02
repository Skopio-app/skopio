// import { index, route, type RouteConfig } from "@react-router/dev/routes";

import { createBrowserRouter } from "react-router";
import App from "./App";
import { Home } from "./home";
import { ExtensionIfrane } from "./extension-iframe";

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
        element: <Home />,
      },
      {
        path: "app/extension",
        element: <ExtensionIfrane />,
      },
    ],
  },
]);

export default router;
