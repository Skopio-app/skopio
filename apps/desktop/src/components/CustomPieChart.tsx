import { ResponsivePieCanvas } from "@nivo/pie";
import { useMemo } from "react";
import { formatDuration } from "@/utils/time";
import { useChartColor, useCssVarColor } from "@/hooks/useChartColor";
import { useElementSize } from "@/hooks/useElementSize";

interface CustomPieChartProps {
  data: {
    id: string;
    label: string;
    value: number;
  }[];
}

const CustomPieChart: React.FC<CustomPieChartProps> = ({ data }) => {
  const chartData = useMemo(
    () => data.sort((a, b) => b.value - a.value),
    [data],
  );

  const { getColorForKey } = useChartColor();
  const linkTextColor = useCssVarColor("--muted-foreground");

  const { ref, width } = useElementSize<HTMLDivElement>();

  const enableArcLinkLabels = width >= 420;

  const margin = useMemo(() => {
    if (width < 360) return { top: 44, right: 44, bottom: 44, left: 44 };
    if (width < 520) return { top: 36, right: 56, bottom: 36, left: 56 };
    return { top: 30, right: 70, bottom: 30, left: 70 };
  }, [width]);

  if (!chartData.length) {
    return (
      <div className="h-[220px] w-full flex items-center justify-center text-sm text-muted-foreground">
        No data available
      </div>
    );
  }

  return (
    <div className="h-[200px] w-full flex">
      <div ref={ref} className="flex-1 min-w-0">
        <ResponsivePieCanvas
          data={chartData}
          margin={margin}
          innerRadius={0.5}
          padAngle={0.6}
          cornerRadius={2}
          activeOuterRadiusOffset={8}
          arcLinkLabelsSkipAngle={5}
          arcLinkLabelsTextColor={linkTextColor}
          arcLinkLabelsThickness={2}
          colors={(bar) => getColorForKey(String(bar.id))}
          arcLinkLabelsDiagonalLength={width < 420 ? 8 : 12}
          arcLinkLabelsStraightLength={width < 420 ? 8 : 12}
          arcLinkLabelsTextOffset={width < 420 ? 4 : 6}
          arcLinkLabelsColor={{ from: "color" }}
          arcLabelsSkipAngle={10}
          arcLabelsTextColor={{ from: "color", modifiers: [["darker", 2]] }}
          legends={undefined}
          enableArcLabels={false}
          enableArcLinkLabels={enableArcLinkLabels}
          tooltip={({ datum }) => {
            const time = formatDuration(datum.value);
            return (
              <div className="min-w-36 rounded-md border border-border bg-background px-3 py-2 text-sm shadow-md text-muted-foreground flex items-center gap-2">
                <span
                  className="w-2.5 h-2.5 rounded-full inline-block"
                  style={{ backgroundColor: datum.color }}
                />
                <div className="flex flex-col">
                  <span className="font-semibold">{datum.id}</span>
                  <span className="text-xs text-muted-foreground">{time}</span>
                </div>
              </div>
            );
          }}
        />
      </div>

      <div className="basis-[clamp(10rem,30%,18rem)] shrink-0 pr-2 pl-3 overflow-y-auto max-h-[300px] space-y-2 text-sm scroll-hidden">
        {chartData.map((d) => (
          <div key={d.id} className="flex items-center justify-between gap-3">
            <div className="flex items-center gap-2 min-w-0 flex-1">
              <span
                className="w-3 h-3 rounded-full inline-block shrink-0"
                style={{ backgroundColor: getColorForKey(d.id) }}
              />
              <span
                title={d.label}
                className="flex-1 min-w-0 truncate text-foreground text-xs"
              >
                {d.label}
              </span>
            </div>

            <span className="shrink-0 truncate text-xs text-muted-foreground max-w-[5rem] text-right">
              {formatDuration(d.value)}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default CustomPieChart;
