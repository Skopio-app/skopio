import { useEffect } from "react";
import { useNavigate } from "react-router";
import { builtinExtensionRegistry } from "@/extensions/registry";
import { LAST_ACTIVE_TAB } from "@/utils/constants";
import LoadingPage from "./loading/LoadingPage";
import { useServerStatus } from "@/hooks/useServerStatus";

const RedirectToTab = () => {
  const navigate = useNavigate();
  const status = useServerStatus();

  useEffect(() => {
    if (status.state !== "running") return;

    const savedId = localStorage.getItem(LAST_ACTIVE_TAB);
    const allTabs = builtinExtensionRegistry.getTabExtensions();
    const defaultTab = allTabs[0];

    const isValid = allTabs.some((ext) => ext.manifest.id === savedId);
    const targetId = isValid ? savedId : defaultTab.manifest.id;

    if (targetId) {
      navigate(`/tab/${targetId}`, { replace: true });
    }
  }, [navigate, status.state]);

  if (status.state !== "running") {
    return <LoadingPage />;
  }
};

export default RedirectToTab;
