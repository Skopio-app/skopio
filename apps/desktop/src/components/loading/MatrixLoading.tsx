import { useCssVarColor } from "@/hooks/useChartColor";
import { useEffect, useRef, useState } from "react";
import {
  matrixColumns,
  matrixFrameRate,
  matrixFramesByTheme,
  matrixLevels,
  matrixRenderBounds,
} from "./matrix/generatedFrames";

type MatrixLoadingProps = {
  className?: string;
};

const MatrixLoading = ({ className }: MatrixLoadingProps) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const frameIndexRef = useRef(0);
  const foreground = useCssVarColor("--foreground");
  const themeMode = useResolvedThemeMode();
  const matrixFrames = matrixFramesByTheme[themeMode];

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }

    const resizeCanvas = () => {
      const dpr = window.devicePixelRatio || 1;
      const rect = canvas.getBoundingClientRect();
      const width = Math.max(1, Math.round(rect.width));
      const height = Math.max(1, Math.round(rect.height));

      if (canvas.width !== width * dpr || canvas.height !== height * dpr) {
        canvas.width = width * dpr;
        canvas.height = height * dpr;
      }

      context.setTransform(dpr, 0, 0, dpr, 0, 0);
      drawFrame(
        context,
        matrixFrames[frameIndexRef.current % matrixFrames.length],
        width,
        height,
        foreground,
      );
    };

    resizeCanvas();

    const resizeObserver = new ResizeObserver(resizeCanvas);
    resizeObserver.observe(canvas);

    return () => resizeObserver.disconnect();
  }, [foreground, matrixFrames]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }

    const drawCurrentFrame = () => {
      const rect = canvas.getBoundingClientRect();
      drawFrame(
        context,
        matrixFrames[frameIndexRef.current % matrixFrames.length],
        Math.max(1, Math.round(rect.width)),
        Math.max(1, Math.round(rect.height)),
        foreground,
      );
    };

    if (
      matrixFrames.length <= 1 ||
      window.matchMedia("(prefers-reduced-motion: reduce)").matches
    ) {
      drawCurrentFrame();
      return;
    }

    let animationFrame = 0;
    let previousFrameAt = performance.now();
    const frameDuration = 1000 / matrixFrameRate;

    const animate = (now: number) => {
      if (now - previousFrameAt >= frameDuration) {
        frameIndexRef.current =
          (frameIndexRef.current + 1) % matrixFrames.length;
        drawCurrentFrame();
        previousFrameAt = now;
      }

      animationFrame = window.requestAnimationFrame(animate);
    };

    animationFrame = window.requestAnimationFrame(animate);

    return () => window.cancelAnimationFrame(animationFrame);
  }, [foreground, matrixFrames]);

  return (
    <div
      className={[
        "relative w-[min(92vw,820px)] overflow-hidden rounded-md border border-border/60 bg-background/40 p-3 shadow-sm",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
      style={{
        aspectRatio: `${matrixRenderBounds.width} / ${matrixRenderBounds.height}`,
      }}
    >
      <canvas
        ref={canvasRef}
        className="block h-full w-full"
        role="img"
        aria-label="Skopio loading animation"
      />
    </div>
  );
};

function useResolvedThemeMode() {
  const [themeMode, setThemeMode] = useState<"light" | "dark">(() =>
    document.documentElement.classList.contains("dark") ? "dark" : "light",
  );

  useEffect(() => {
    const updateThemeMode = () => {
      setThemeMode(
        document.documentElement.classList.contains("dark") ? "dark" : "light",
      );
    };

    updateThemeMode();

    const observer = new MutationObserver(updateThemeMode);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    return () => observer.disconnect();
  }, []);

  return themeMode;
}

function drawFrame(
  context: CanvasRenderingContext2D,
  frame: string,
  width: number,
  height: number,
  color: string,
) {
  const cellWidth = width / matrixRenderBounds.width;
  const cellHeight = height / matrixRenderBounds.height;

  context.clearRect(0, 0, width, height);
  context.fillStyle = color;

  for (
    let row = matrixRenderBounds.y;
    row < matrixRenderBounds.y + matrixRenderBounds.height;
    row += 1
  ) {
    for (
      let column = matrixRenderBounds.x;
      column < matrixRenderBounds.x + matrixRenderBounds.width;
      column += 1
    ) {
      const index = row * matrixColumns + column;
      const level = decodeLevel(frame.charCodeAt(index));

      if (level <= 0) {
        continue;
      }

      const normalized = level / Math.max(1, matrixLevels - 1);
      context.globalAlpha = 0.1 + normalized * 0.9;
      context.fillRect(
        (column - matrixRenderBounds.x) * cellWidth,
        (row - matrixRenderBounds.y) * cellHeight,
        cellWidth,
        cellHeight,
      );
    }
  }

  context.globalAlpha = 1;
}

function decodeLevel(charCode: number) {
  if (charCode >= 48 && charCode <= 57) {
    return charCode - 48;
  }

  if (charCode >= 97 && charCode <= 122) {
    return charCode - 87;
  }

  return 0;
}

export default MatrixLoading;
