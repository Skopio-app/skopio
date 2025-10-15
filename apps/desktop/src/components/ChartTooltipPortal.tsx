import { useLayoutEffect, useRef } from "react";
import { createPortal } from "react-dom";

const tooltipRoot = typeof window !== "undefined" ? document.body : null;

const ChartTooltipPortal: React.FC<{
  children: React.ReactNode;
  style?: React.CSSProperties;
}> = ({ children, style }) => {
  const ref = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    if (!ref.current || !style || typeof window === "undefined") return;

    const el = ref.current;
    el.style.visibility = "hidden";

    el.style.position = "fixed";
    el.style.pointerEvents = "none";
    el.style.transition = "opacity 100ms ease-out";
    el.style.zIndex = String(style.zIndex ?? 999);

    const reqTop =
      typeof style.top === "number"
        ? style.top
        : parseFloat(String(style.top ?? 0));
    const reqLeft =
      typeof style.left === "number"
        ? style.left
        : parseFloat(String(style.left ?? 0));

    el.style.top = `${reqTop}px`;
    el.style.left = `${reqLeft}px`;

    const rect = el.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    let top = reqTop;
    let left = reqLeft;

    if (left + rect.width > vw) left = Math.max(0, vw - rect.width - 8);
    if (top + rect.height > vh) top = Math.max(0, top - rect.height - 16);

    el.style.top = `${top}px`;
    el.style.left = `${left}px`;

    el.style.visibility = "visible";
  }, [style]);

  if (!tooltipRoot || !style) return null;

  return createPortal(
    <div
      ref={ref}
      style={{
        position: "fixed",
        pointerEvents: "none",
        zIndex: 999,
        visibility: "hidden",
        maxHeight: "50vh",
        ...style,
      }}
    >
      {children}
    </div>,
    tooltipRoot,
  );
};

export default ChartTooltipPortal;
