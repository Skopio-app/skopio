import { useRef, useEffect, useMemo } from "react";
import { formatDuration } from "@/utils/time";
import { useChartColor, useCssVarColor } from "@/hooks/useChartColor";
import { truncateValue } from "@/utils/data";
import * as echarts from "echarts/core";
import { CustomChart, CustomSeriesOption } from "echarts/charts";
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
  CustomChart,
  GridComponent,
  TooltipComponent,
  DatasetComponent,
  CanvasRenderer,
]);

type ChartOption = ComposeOption<
  | CustomSeriesOption
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
const MAX_BAR_WIDTH = 42;
const BAR_WIDTH_RATIO = 0.7;
const BAR_BORDER_RADIUS = 2;
const TOOLTIP_GAP = 8;
const TOOLTIP_VIEWPORT_MARGIN = 8;

type StackSegment = {
  key: string;
  value: number;
  color: string;
  start: number;
  end: number;
  rowIndex: number;
};

type StackedRow = {
  label: string;
  total: number;
  segments: StackSegment[];
};

type FlatStackSegment = StackSegment & {
  label: string;
};

const StackedBarChart: React.FC<StackedBarChartProps> = ({
  data,
  keys,
  bucketLabel = "Days",
  axisBottom = true,
  axisLeft = false,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<ECharts | null>(null);
  const lastResizeRef = useRef({ width: 0, height: 0, rafId: 0 });
  const lastAppliedOptionSignatureRef = useRef("");
  const lastOptionDataSignatureRef = useRef("");
  const tooltipVisibleRef = useRef(false);

  const { getColorForKey } = useChartColor();
  const axisTextColor = useCssVarColor("--muted-foreground");
  const backgroundColor = useCssVarColor("--background");
  const foregroundColor = useCssVarColor("--foreground");
  const mutedForegroundColor = useCssVarColor("--muted-foreground");
  const borderColor = useCssVarColor("--border");
  const gridColor = useCssVarColor("--input");
  const hoverBackground = `color-mix(in oklch, ${borderColor} 30%, transparent)`;

  const { stackedRows, flatSegments } = useMemo<{
    stackedRows: StackedRow[];
    flatSegments: FlatStackSegment[];
  }>(() => {
    return data.reduce<{
      stackedRows: StackedRow[];
      flatSegments: FlatStackSegment[];
    }>(
      (acc, row, rowIndex) => {
        let offset = 0;
        const segments = keys
          .map((key, index) => ({
            key,
            index,
            value: Number(row[key] ?? 0),
            color: getColorForKey(key),
          }))
          .filter((segment) => segment.value > 0)
          .sort((a, b) => a.value - b.value || a.index - b.index)
          .map(({ key, value, color }) => {
            const segment = {
              key,
              value,
              color,
              start: offset,
              end: offset + value,
              rowIndex,
            };
            offset += value;
            return segment;
          });

        const stackedRow = {
          label: String(row.label),
          total: offset,
          segments,
        };

        acc.stackedRows.push(stackedRow);
        acc.flatSegments.push(
          ...segments.map((segment) => ({
            ...segment,
            label: stackedRow.label,
          })),
        );

        return acc;
      },
      { stackedRows: [], flatSegments: [] },
    );
  }, [data, getColorForKey, keys]);

  const option = useMemo<ChartOption>(() => {
    return {
      animation: false,

      grid: {
        top: 20,
        right: 30,
        bottom: axisBottom ? 50 : 20,
        left: axisLeft ? 90 : 50,
        outerBoundsMode: "same",
        outerBoundsContain: "axisLabel",
      },

      tooltip: {
        trigger: "item",
        confine: false,
        appendTo: "body",
        alwaysShowContent: false,
        enterable: false,
        hideDelay: 0,
        transitionDuration: 0,
        displayTransition: false,
        position: (point, _params, _dom, rect, size) => {
          const chartRect = containerRef.current?.getBoundingClientRect();
          const viewport = window.visualViewport;
          const viewportWidth =
            viewport?.width ?? document.documentElement.clientWidth;
          const viewportHeight =
            viewport?.height ?? document.documentElement.clientHeight;
          const viewportLeft = viewport?.offsetLeft ?? 0;
          const viewportTop = viewport?.offsetTop ?? 0;
          const contentWidth = Math.min(
            size.contentSize[0] || 320,
            Math.max(1, viewportWidth - TOOLTIP_VIEWPORT_MARGIN * 2),
          );
          const contentHeight = Math.min(
            size.contentSize[1] || 384,
            Math.max(1, viewportHeight - TOOLTIP_VIEWPORT_MARGIN * 2),
          );

          if (!chartRect) {
            return [
              Math.max(TOOLTIP_VIEWPORT_MARGIN, point[0] + TOOLTIP_GAP),
              Math.max(TOOLTIP_VIEWPORT_MARGIN, point[1] - contentHeight / 2),
            ];
          }

          const chartLeft = chartRect.left + viewportLeft;
          const chartTop = chartRect.top + viewportTop;
          const viewportRight = viewportLeft + viewportWidth;
          const viewportBottom = viewportTop + viewportHeight;
          const itemRect = rect as
            | { x: number; y: number; width: number; height: number }
            | undefined;
          const anchorX = itemRect ? itemRect.x + itemRect.width : point[0];
          const anchorY = itemRect
            ? itemRect.y + itemRect.height / 2
            : point[1];
          const hoveredViewportX = chartLeft + anchorX;
          const hoveredViewportY = chartTop + anchorY;
          const minX = viewportLeft + TOOLTIP_VIEWPORT_MARGIN;
          const maxX = viewportRight - contentWidth - TOOLTIP_VIEWPORT_MARGIN;
          const minY = viewportTop + TOOLTIP_VIEWPORT_MARGIN;
          const maxY = viewportBottom - contentHeight - TOOLTIP_VIEWPORT_MARGIN;

          let viewportX: number;

          if (
            hoveredViewportX + TOOLTIP_GAP + contentWidth <=
            viewportRight - TOOLTIP_VIEWPORT_MARGIN
          ) {
            viewportX = hoveredViewportX + TOOLTIP_GAP;
          } else if (
            hoveredViewportX - TOOLTIP_GAP - contentWidth >=
            viewportLeft + TOOLTIP_VIEWPORT_MARGIN
          ) {
            viewportX = hoveredViewportX - contentWidth - TOOLTIP_GAP;
          } else if (hoveredViewportX > viewportLeft + viewportWidth / 2) {
            viewportX = minX;
          } else {
            viewportX = maxX;
          }

          const viewportY = Math.min(
            Math.max(minY, hoveredViewportY - contentHeight / 2),
            maxY,
          );

          return [viewportX - chartLeft, viewportY - chartTop];
        },
        backgroundColor: backgroundColor,
        borderColor: borderColor,
        borderWidth: 1,
        padding: [12, 16],
        textStyle: {
          color: foregroundColor,
          fontSize: 12,
        },
        extraCssText: [
          "border-radius:6px",
          "box-shadow:0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
        ].join(";"),

        formatter: (params) => {
          const items = Array.isArray(params) ? params : [params];
          const dataIndex = items[0]?.dataIndex ?? -1;
          const hoveredSegment = flatSegments[dataIndex];
          const rowIndex = hoveredSegment?.rowIndex ?? -1;
          const row = stackedRows[rowIndex];

          if (!hoveredSegment || !row) {
            return "";
          }

          tooltipVisibleRef.current = true;

          const visibleItems = (row?.segments ?? [])
            .slice()
            .sort((a, b) => b.value - a.value)
            .slice(0, MAX_TOOLTIP_ENTRIES);

          const title = row?.label ?? String(items[0]?.name ?? "");

          const rows = visibleItems
            .map((item) => {
              const isHighlighted =
                hoveredSegment?.rowIndex === item.rowIndex &&
                hoveredSegment.key === item.key;
              const name = truncateValue(item.key, 25);
              const duration = formatDuration(item.value);

              return `
                <div style="
                  display:flex;
                  align-items:center;
                  gap:8px;
                  padding:4px 6px;
                  margin:1px -6px;
                  border-radius:4px;
                  border-left:3px solid ${isHighlighted ? item.color : "transparent"};
                  background:${isHighlighted ? hoverBackground : "transparent"};
                  font-weight:${isHighlighted ? 600 : 400};
                ">
                  <span style="
                    width:10px;
                    height:10px;
                    border-radius:999px;
                    background:${item.color};
                    display:inline-block;
                    flex-shrink:0;
                  "></span>

                <span style="
                  flex:1;
                  min-width:0;
                  color:${foregroundColor};
                ">${name}</span>

                <span style="
                  color:${mutedForegroundColor};
                  white-space:nowrap;
                ">${duration}</span>
              </div>
            `;
            })
            .join("");

          return `
                <div class="scroll-hidden" style="
                    min-width:200px;
                    max-width:min(320px, calc(100vw - 24px));
                    max-height:min(384px, calc(100vh - 24px));
                    overflow-y:auto;
                    overscroll-behavior:contain;
                    scrollbar-width: none;
                    -ms-overflow-style: none;
                    color:${foregroundColor};
                ">
                  <div style="font-weight:600;margin-bottom:4px;color:${foregroundColor};">
                    ${title}
                  </div>
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

      series: {
        type: "custom",
        name: bucketLabel,
        coordinateSystem: "cartesian2d",
        data: flatSegments.map((segment) => [
          segment.rowIndex,
          segment.end,
          segment.start,
          segment.value,
        ]),
        encode: {
          x: 0,
          y: 1,
        },
        renderItem: (params, api) => {
          const segment = flatSegments[params.dataIndex];
          if (!segment) return null;

          const categorySize = api.size?.([1, 0]);
          const categoryWidth = Array.isArray(categorySize)
            ? Number(categorySize[0])
            : MAX_BAR_WIDTH;
          const barWidth = Math.min(
            MAX_BAR_WIDTH,
            Math.max(1, categoryWidth * BAR_WIDTH_RATIO),
          );
          const coordSys = params.coordSys as unknown as {
            x: number;
            y: number;
            width: number;
            height: number;
          };

          const start = api.coord([segment.rowIndex, segment.start]);
          const end = api.coord([segment.rowIndex, segment.end]);
          const height = Math.max(0, start[1] - end[1]);
          const shape = echarts.graphic.clipRectByRect(
            {
              x: start[0] - barWidth / 2,
              y: end[1],
              width: barWidth,
              height,
            },
            {
              x: coordSys.x,
              y: coordSys.y,
              width: coordSys.width,
              height: coordSys.height,
            },
          );

          if (!shape) return null;

          return {
            type: "rect",
            name: `${segment.label}:${segment.key}`,
            focus: "self",
            blurScope: "coordinateSystem",
            shape: {
              ...shape,
              r: BAR_BORDER_RADIUS,
            },
            style: {
              fill: segment.color,
              stroke: backgroundColor,
              lineWidth: 0.5,
            },
            emphasis: {
              style: {
                stroke: borderColor,
                lineWidth: 1,
              },
            },
            blur: {
              style: {
                opacity: 0.32,
              },
            },
          };
        },
      },
    };
  }, [
    axisBottom,
    axisLeft,
    axisTextColor,
    backgroundColor,
    borderColor,
    foregroundColor,
    mutedForegroundColor,
    bucketLabel,
    data,
    flatSegments,
    gridColor,
    hoverBackground,
    stackedRows,
  ]);

  const optionDataSignature = useMemo(
    () =>
      stackedRows
        .map(
          (row) =>
            `${row.label}:${row.total}:${row.segments
              .map(
                (segment) =>
                  `${segment.key}:${segment.start}:${segment.end}:${segment.value}`,
              )
              .join(",")}`,
        )
        .join("|"),
    [stackedRows],
  );

  const optionSignature = useMemo(
    () =>
      [
        optionDataSignature,
        axisBottom,
        axisLeft,
        axisTextColor,
        backgroundColor,
        borderColor,
        bucketLabel,
        foregroundColor,
        gridColor,
        hoverBackground,
        mutedForegroundColor,
      ].join("::"),
    [
      axisBottom,
      axisLeft,
      axisTextColor,
      backgroundColor,
      borderColor,
      bucketLabel,
      foregroundColor,
      gridColor,
      hoverBackground,
      mutedForegroundColor,
      optionDataSignature,
    ],
  );

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

    const chart = chartRef.current as ECharts & {
      on: (eventName: string, handler: (params: unknown) => void) => void;
      off: (eventName: string, handler: (params: unknown) => void) => void;
    };
    const handleShowTip = () => {
      tooltipVisibleRef.current = true;
    };
    const handleHideTip = () => {
      tooltipVisibleRef.current = false;
    };

    chart.on("showTip", handleShowTip);
    chart.on("hideTip", handleHideTip);

    return () => {
      chart.off("showTip", handleShowTip);
      chart.off("hideTip", handleHideTip);
    };
  }, []);

  useEffect(() => {
    if (!chartRef.current) return;

    const previousDataSignature = lastOptionDataSignatureRef.current;
    const previousOptionSignature = lastAppliedOptionSignatureRef.current;
    const didDataChange = previousDataSignature !== optionDataSignature;
    const didOptionChange = previousOptionSignature !== optionSignature;

    if (!didOptionChange) return;

    if (didDataChange) {
      chartRef.current.dispatchAction({ type: "hideTip" });
      chartRef.current.dispatchAction({ type: "downplay" });
    }

    chartRef.current.setOption(option, {
      notMerge: true,
      lazyUpdate: false,
    });
    lastOptionDataSignatureRef.current = optionDataSignature;
    lastAppliedOptionSignatureRef.current = optionSignature;
  }, [
    flatSegments.length,
    keys.length,
    option,
    optionDataSignature,
    optionSignature,
    stackedRows.length,
  ]);

  useEffect(() => {
    if (!containerRef.current || !chartRef.current) return;

    const container = containerRef.current;
    const hideTooltip = () => {
      tooltipVisibleRef.current = false;
      chartRef.current?.dispatchAction({ type: "hideTip" });
      chartRef.current?.dispatchAction({ type: "downplay" });
    };
    const isPointerInsideChart = (event: MouseEvent | PointerEvent) => {
      const rect = container.getBoundingClientRect();
      return (
        event.clientX >= rect.left &&
        event.clientX <= rect.right &&
        event.clientY >= rect.top &&
        event.clientY <= rect.bottom
      );
    };
    const handleMouseLeave = () => {
      hideTooltip();
    };
    const handlePointerLeave = () => {
      hideTooltip();
    };
    const handleDocumentPointerMove = (event: PointerEvent) => {
      if (!tooltipVisibleRef.current || isPointerInsideChart(event)) return;

      hideTooltip();
    };
    const handleDocumentMouseMove = (event: MouseEvent) => {
      if (!tooltipVisibleRef.current || isPointerInsideChart(event)) return;

      hideTooltip();
    };
    const handleWindowBlur = () => {
      if (!tooltipVisibleRef.current) return;

      hideTooltip();
    };
    const handleDocumentScroll = () => {
      if (!tooltipVisibleRef.current) return;

      hideTooltip();
    };

    container.addEventListener("mouseleave", handleMouseLeave);
    container.addEventListener("pointerleave", handlePointerLeave);
    document.addEventListener("pointermove", handleDocumentPointerMove, true);
    document.addEventListener("mousemove", handleDocumentMouseMove, true);
    document.addEventListener("scroll", handleDocumentScroll, true);
    window.addEventListener("blur", handleWindowBlur);

    const resizeState = lastResizeRef.current;
    const resizeObserver = new ResizeObserver(([entry]) => {
      const { width, height } = entry.contentRect;

      if (
        Math.round(width) === resizeState.width &&
        Math.round(height) === resizeState.height
      ) {
        return;
      }

      resizeState.width = Math.round(width);
      resizeState.height = Math.round(height);

      if (resizeState.rafId) cancelAnimationFrame(resizeState.rafId);
      resizeState.rafId = requestAnimationFrame(() => {
        chartRef.current?.resize();
        resizeState.rafId = 0;
      });
    });

    resizeObserver.observe(containerRef.current);

    return () => {
      container.removeEventListener("mouseleave", handleMouseLeave);
      container.removeEventListener("pointerleave", handlePointerLeave);
      document.removeEventListener(
        "pointermove",
        handleDocumentPointerMove,
        true,
      );
      document.removeEventListener("mousemove", handleDocumentMouseMove, true);
      document.removeEventListener("scroll", handleDocumentScroll, true);
      window.removeEventListener("blur", handleWindowBlur);
      if (resizeState.rafId) {
        cancelAnimationFrame(resizeState.rafId);
        resizeState.rafId = 0;
      }
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
