import { createBrowserRouter } from "react-router";
import App from "./App";
import TabExtensionPage from "./pages/TabExtensionPage";
import DashboardLayout from "./components/Dashboard";
import RedirectToTab from "./components/RedirectToTab";
import ProjectDetails from "./extensions/projects-view/ProjectDetails";

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
            children: [
              {
                path: "projects/:projectId",
                element: <ProjectDetails />,
              },
            ],
          },
        ],
      },
    ],
  },
]);

export default router;
