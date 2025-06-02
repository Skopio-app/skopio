import { useSearchParams } from "react-router-dom";

export const ExtensionIfrane = () => {
  const [params] = useSearchParams();
  const url = params.get("url");
  const extPath = params.get("extPath");

  if (!url) {
    return <p>Missing extension</p>;
  }

  return (
    <iframe
      src={url}
      title={extPath ?? "extension"}
      style={{
        width: "100%",
        height: "100%",
        border: "none",
      }}
    />
  );
};
