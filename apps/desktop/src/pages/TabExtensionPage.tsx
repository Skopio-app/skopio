import { Outlet, useMatch, useParams } from "react-router";
import { builtinExtensionRegistry } from "@/extensions/registry";
import { TabExtensionHost } from "@/pages/TabExtensionHost";

const TabExtensionPage = () => {
  const { id } = useParams();
  const isProjectView = useMatch("/tab/:id/projects/:projectId");
  const extension = builtinExtensionRegistry.getExtensionById(id ?? "");

  if (!extension || !extension.component) {
    return (
      <div className="p-4 text-red-500">Extension not found or invalid</div>
    );
  }

  return (
    <div className="h-full w-full">
      {isProjectView ? <Outlet /> : <TabExtensionHost extension={extension} />}
    </div>
  );
};

export default TabExtensionPage;
