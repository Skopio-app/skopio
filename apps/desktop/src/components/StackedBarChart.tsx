import { useRef, useEffect, useMemo } from "react";
import { formatDuration } from "@/utils/time";
import { useChartColor, useCssVarColor } from "@/hooks/useChartColor";
import { truncateValue } from "@/utils/data";
import * as echarts from "echarts/core";
import { BarChart, BarSeriesOption } from "echarts/charts";
import {
  GridComponent,
  TooltipComponent,
  DatasetComponent,
  GridComponentOption,
  TooltipComponentOption,
  DatasetComponentOption,
} from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { ECharts, ComposeOption } from "echarts/core";

echarts.use([
  BarChart,
  GridComponent,
  TooltipComponent,
  DatasetComponent,
  CanvasRenderer,
]);

type ChartOption = ComposeOption<
  | BarSeriesOption
  | GridComponentOption
  | TooltipComponentOption
  | DatasetComponentOption
>;

interface StackedBarChartProps {
  data: {
    label: string;
    [key: string]: string | number;
  }[];
  keys: string[];
  bucketLabel?: string;
  axisBottom?: boolean;
  axisLeft?: boolean;
}

const MAX_TOOLTIP_ENTRIES = 10;

const escapeHtml = (value: string) =>
  value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");

const StackedBarChart: React.FC<StackedBarChartProps> = ({
  data,
  keys,
  bucketLabel = "Time",
  axisBottom = true,
  axisLeft = false,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<ECharts | null>(null);

  const { getColorForKey } = useChartColor();
  const axisTextColor = useCssVarColor("--muted-foreground");
  const gridColor = useCssVarColor("--input");

  const option = useMemo<ChartOption>(() => {
    return {
      animation: false,

      grid: {
        top: 20,
        right: 30,
        bottom: axisBottom ? 50 : 20,
        left: axisLeft ? 90 : 50,
        containLabel: true, // deprecated
      },

      tooltip: {
        trigger: "axis",
        axisPointer: {
          type: "shadow",
        },
        order: "seriesAsc",
        confine: true,
        backgroundColor: "var(--background)",
        borderColor: "var(--border)",
        borderWidth: 1,
        padding: [12, 16],
        textStyle: {
          color: "var(--foreground)",
          fontSize: 12,
        },
        extraCssText: [
          "border-radius:6px",
          "box-shadow:0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
        ].join(";"),
        formatter: (params) => {
          const items = Array.isArray(params) ? params : [params];

          const visibleItems = items
            .map((item) => ({
              name: String(item.seriesName ?? ""),
              value: Number(item.value ?? 0),
              color: String(item.color ?? ""),
            }))
            .filter((item) => item.value > 0)
            .sort((a, b) => b.value - a.value)
            .slice(0, MAX_TOOLTIP_ENTRIES);

          const title = escapeHtml(String(items[0]?.name ?? ""));

          const rows = visibleItems
            .map((item) => {
              const name = escapeHtml(truncateValue(item.name, 25));
              const duration = escapeHtml(formatDuration(item.value));

              return `
                <div style="display:flex;align-items:center;gap:8px;padding:2px 0;">
                  <span style="width:10px;height:10px;border-radius:999px;background:${item.color};display:inline-block;flex-shrink:0;"></span>
                  <span style="flex:1;min-width:0;color:var(--foreground);">${name}</span>
                  <span style="color:var(--muted-foreground);white-space:nowrap;">${duration}</span>
                </div>
              `;
            })
            .join("");

          return `
            <div style="min-width:200px;max-width:320px;max-height:384px;overflow-y:auto;color:var(--foreground);">
              <div style="font-weight:600;margin-bottom:4px;color:var(--foreground);">${title}</div>
              ${rows}
            </div>
          `;
        },
      },
      xAxis: {
        type: "category",
        data: data.map((row) => row.label),
        name: axisBottom ? bucketLabel : undefined,
        nameLocation: "middle",
        nameGap: 32,
        axisLabel: {
          color: axisTextColor,
          fontSize: 11,
        },
        axisLine: {
          lineStyle: {
            color: axisTextColor,
          },
        },
        axisTick: {
          lineStyle: {
            color: axisTextColor,
          },
        },
      },

      yAxis: {
        type: "value",
        show: axisLeft,
        name: axisLeft ? bucketLabel : undefined,
        nameLocation: "middle",
        nameGap: 70,
        axisLabel: {
          color: axisTextColor,
          formatter: (value: number) => formatDuration(value),
        },
        splitLine: {
          lineStyle: {
            color: gridColor,
            type: "dashed",
          },
        },
      },

      series: keys.map((key): BarSeriesOption => {
        return {
          type: "bar",
          name: key,
          data: data.map((row) => Number(row[key] ?? 0)),
          stack: "total",
          stackOrder: "seriesAsc",
          color: getColorForKey(key),
          barMaxWidth: 42,
          itemStyle: {
            borderRadius: [2, 2, 0, 0],
          },
          emphasis: {
            focus: "series",
          },
        };
      }),
    };
  }, [
    axisBottom,
    axisLeft,
    axisTextColor,
    bucketLabel,
    data,
    getColorForKey,
    gridColor,
    keys,
  ]);

  useEffect(() => {
    if (!containerRef.current) return;

    chartRef.current = echarts.init(containerRef.current, undefined, {
      renderer: "canvas",
    });

    return () => {
      chartRef.current?.dispose();
      chartRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (!chartRef.current) return;

    chartRef.current.setOption(option, {
      notMerge: true,
      lazyUpdate: true,
    });
  }, [option]);

  useEffect(() => {
    if (!containerRef.current || !chartRef.current) return;

    const resizeObserver = new ResizeObserver(() => {
      chartRef.current?.resize();
    });

    resizeObserver.observe(containerRef.current);

    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  if (!data.length) {
    return (
      <p className="h-55 w-full flex items-center justify-center text-sm text-muted-foreground">
        No data available
      </p>
    );
  }

  return <div ref={containerRef} className="h-55 w-full" />;
};

export default StackedBarChart;
