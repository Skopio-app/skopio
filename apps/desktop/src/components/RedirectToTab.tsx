import { useEffect } from "react";
import { useNavigate } from "react-router";
import { builtinExtensionRegistry } from "@/extensions/registry";
import { LAST_ACTIVE_TAB } from "@/utils/constants";

const RedirectToTab = () => {
  const navigate = useNavigate();

  useEffect(() => {
    const savedId = localStorage.getItem(LAST_ACTIVE_TAB);
    const allTabs = builtinExtensionRegistry.getTabExtensions();
    const defaultTab = allTabs[0];

    const isValid = allTabs.some((ext) => ext.manifest.id === savedId);
    const targetId = isValid ? savedId : defaultTab.manifest.id;

    if (targetId) {
      navigate(`/tab/${targetId}`, { replace: true });
    }
  }, [navigate]);

  return null;
};

export default RedirectToTab;
