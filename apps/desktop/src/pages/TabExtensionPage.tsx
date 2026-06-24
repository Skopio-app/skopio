import { Outlet, useMatch, useParams } from "react-router";
import { builtinExtensionRegistry } from "@/extensions/registry";
import { TabExtensionHost } from "@/pages/TabExtensionHost";
import { SubtleScrollArea } from "@/components/SubtleScrollArea";

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
    <div className="h-full min-h-0 w-full">
      {isProjectView ? (
        <SubtleScrollArea className="h-full w-full">
          <Outlet />
        </SubtleScrollArea>
      ) : (
        <TabExtensionHost extension={extension} />
      )}
    </div>
  );
};

export default TabExtensionPage;
