import { useParams } from "react-router";
import { builtinExtensionRegistry } from "../extensions/registry";
import { TabExtensionHost } from "./TabExtensionHost";

const TabExtensionPage = () => {
  const { id } = useParams();
  const extension = builtinExtensionRegistry.getExtensionById(id ?? "");

  if (!extension || !extension.component) {
    return (
      <div className="p-4 text-red-500">Extension not found or invalid</div>
    );
  }

  return <TabExtensionHost extension={extension} />;
};

export default TabExtensionPage;
