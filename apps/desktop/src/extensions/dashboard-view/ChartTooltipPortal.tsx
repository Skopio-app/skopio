import { useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

const tooltipRoot = typeof window !== "undefined" ? document.body : null;

const ChartTooltipPortal: React.FC<{
  children: React.ReactNode;
  style?: React.CSSProperties;
}> = ({ children, style }) => {
  const ref = useRef<HTMLDivElement>(null);
  const [adjustedStyle, setAdjustedStyle] = useState<React.CSSProperties>();

  useLayoutEffect(() => {
    if (!ref.current || !style) return;

    const tooltipRect = ref.current.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    let top =
      typeof style.top === "number"
        ? style.top
        : parseFloat(String(style.top ?? 0));
    let left =
      typeof style.left === "number"
        ? style.left
        : parseFloat(String(style.left ?? 0));

    // Flip horizontally if overflowing
    if (left + tooltipRect.width > viewportWidth) {
      left = Math.max(0, viewportWidth - tooltipRect.width - 8);
    }

    // Flip vertically if overflowing
    if (top + tooltipRect.height > viewportHeight) {
      top = Math.max(0, top - tooltipRect.height - 16);
    }

    setAdjustedStyle({
      ...style,
      top,
      left,
    });
  }, [style]);

  if (!tooltipRoot || !style) return null;

  return createPortal(
    <div
      ref={ref}
      style={{
        position: "fixed",
        pointerEvents: "none",
        zIndex: 999,
        visibility: adjustedStyle ? "visible" : "hidden",
        transition: "opacity 100ms ease-out",
        maxHeight: Math.min(400, window.innerHeight * 0.5),
        ...adjustedStyle,
      }}
    >
      {children}
    </div>,
    tooltipRoot,
  );
};

export default ChartTooltipPortal;
