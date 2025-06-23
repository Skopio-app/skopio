import { createPortal } from "react-dom";

const tooltipRoot = typeof window !== "undefined" ? document.body : null;

const ChartTooltipPortal: React.FC<{
  children: React.ReactNode;
  style?: React.CSSProperties;
}> = ({ children, style }) => {
  if (!tooltipRoot) return null;

  return createPortal(
    <div
      style={{
        position: "fixed",
        pointerEvents: "none",
        zIndex: 999,
        ...style,
      }}
    >
      {children}
    </div>,
    tooltipRoot,
  );
};

export default ChartTooltipPortal;
