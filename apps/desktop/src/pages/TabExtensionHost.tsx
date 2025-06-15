import { useEffect, useRef } from "react";
import { ExtensionRegistration } from "../extensions/registry";
import { RPCChannel, IframeIO } from "@skopio/rpc";

interface TabExtensionHostProps {
  extension: ExtensionRegistration;
}

export const TabExtensionHost: React.FC<TabExtensionHostProps> = ({
  extension,
}) => {
  const iframeRef = useRef<HTMLIFrameElement>(null);
  // const [channel, setChannel] = useState<RPCChannel | null>(null);

  const Component = extension.component;

  useEffect(() => {
    const iframe = iframeRef.current;
    if (!iframe || !iframe.contentWindow) return;

    const io = new IframeIO(iframe.contentWindow);
    const rpc = new RPCChannel(io, true);

    // setChannel(rpc);

    return () => {
      rpc.destroy();
      // setChannel(null);
    };
  }, [extension]);

  return (
    <div className="w-full h-full overflow-auto">
      <Component />
    </div>
  );
};
