import { useEffect, useState } from "react";
import { Layout } from "react-grid-layout";
import { skopioDB } from "../db/skopioDB";

export const usePersistentLayout = (key: string, defaultLayout: Layout[]) => {
  const [layout, setLayout] = useState<Layout[]>(defaultLayout);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    skopioDB.layouts.get(key).then((entry) => {
      if (entry?.layout) {
        setLayout(entry.layout);
      }
      setLoaded(true);
    });
  }, [key]);

  const saveLayout = (newLayout: Layout[]) => {
    setLayout(newLayout);
    skopioDB.layouts.put({ id: key, layout: newLayout });
  };

  return { layout, saveLayout, loaded };
};
