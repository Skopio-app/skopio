import { convertFileSrc } from "@tauri-apps/api/core";

export const getDevExtensionUrl = ({
  extPath,
  main,
  devMain,
  isDev,
  hmr,
}: {
  extPath: string;
  main: string;
  devMain?: string;
  isDev: boolean;
  hmr: boolean;
}): string => {
  const useDev = isDev && hmr && devMain;

  const baseUrl = useDev ? devMain! : convertFileSrc(main, "ext");

  const finalUrl = `/app/extension?url=${encodeURIComponent(baseUrl)}&extPath=${encodeURIComponent(extPath)}`;

  return finalUrl;
};
