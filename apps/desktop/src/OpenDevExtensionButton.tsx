import { useNavigate } from "react-router-dom";
import { getDevExtensionUrl } from "./utils/load-extension";
import { Button } from "@skopio/ui";

export const OpenDevExtensionButton = () => {
  const navigate = useNavigate();

  const open = () => {
    const url = getDevExtensionUrl({
      extPath: "/Users/samwahome/CodeProjects/skopio-ext-test",
      main: "dist/index.html",
      devMain: "http://localhost:5174",
      isDev: true,
      hmr: true,
    });

    navigate(url);
  };

  return (
    <Button variant="default" onClick={open}>
      Open Dev Extension
    </Button>
  );
};
