import { useColorCache } from "@/stores/useColorCache";
import { useOrdinalColorScale } from "@nivo/colors";
import { useEffect, useState } from "react";

export const useChartColor = () => {
  const getColorScale = useOrdinalColorScale({ scheme: "nivo" }, "id");

  const getColorForKey = (key: string): string => {
    const cachedColor = useColorCache.getState().getColor(key);
    if (cachedColor) return cachedColor;
    const newColor = getColorScale({ id: key });
    useColorCache.getState().setColor(key, newColor);
    return newColor;
  };

  return { getColorForKey };
};

export const useCssVarColor = (varName: string, fallback = "#333") => {
  const [color, setColor] = useState<string>(fallback);

  useEffect(() => {
    const compute = () => {
      const v = getComputedStyle(document.documentElement)
        .getPropertyValue(varName)
        .trim();
      setColor(v || fallback);
    };

    compute();

    const mo = new MutationObserver(compute);
    mo.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    mql.addEventListener?.("change", compute);

    return () => {
      mo.disconnect();
      mql.removeEventListener?.("change", compute);
    };
  }, [varName, fallback]);

  return color;
};
