import { useColorCache } from "@/stores/useColorCache";
import { useOrdinalColorScale } from "@nivo/colors";

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
