import { createBrowserRouter, Navigate } from "react-router";
import App from "@/App";
import TabExtensionPage from "@/pages/TabExtensionPage";
import DashboardLayout from "@/components/Dashboard";
import RedirectToTab from "@/components/RedirectToTab";
import ProjectDetails from "@/extensions/projects-view/ProjectDetails";
import SettingsPage from "@/pages/SettingsPage";
import General from "@/components/settings/views/General";
import Permission from "@/components/settings/views/Permission";
import { ErrorBoundary, ErrorPage } from "@/routes/error";

const router = createBrowserRouter([
  {
    path: "/",
    element: <App />,
    errorElement: <ErrorPage />,
    children: [
      {
        element: (
          <ErrorBoundary>
            <DashboardLayout />
          </ErrorBoundary>
        ),
        errorElement: <ErrorPage />,
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
      {
        path: "settings",
        element: (
          <ErrorBoundary>
            <SettingsPage />
          </ErrorBoundary>
        ),
        errorElement: <ErrorPage />,
        children: [
          { index: true, element: <Navigate to="general" replace /> },
          { path: "general", element: <General /> },
          { path: "permissions", element: <Permission /> },
        ],
      },
    ],
  },
]);

export default router;
