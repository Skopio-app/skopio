import { useCssVarColor } from "@/hooks/useChartColor";
import { useTheme } from "@/utils/theme";
import type { CSSProperties, RefObject } from "react";
import { useEffect, useRef, useState } from "react";
import {
  matrixColumns,
  matrixFrameRate,
  matrixFramesUrl,
  matrixLevels,
  matrixRenderBounds,
} from "./matrix/generatedFrames";

type MatrixLoadingProps = {
  className?: string;
  fit?: "cover" | "stretch";
  height?: CSSProperties["height"];
  style?: CSSProperties;
  width?: CSSProperties["width"];
};

const MATRIX_ASPECT_RATIO =
  matrixRenderBounds.width / matrixRenderBounds.height;
const DEFAULT_MAX_WIDTH = 980;
const MIN_RESPONSIVE_HEIGHT = 120;
const VIEWPORT_INLINE_GUTTER = 32;
const VIEWPORT_BLOCK_RESERVE = 180;

type MatrixThemeMode = "light" | "dark";
type MatrixFramesByTheme = Record<MatrixThemeMode, readonly string[]>;
type MatrixFramesPayload = {
  framesByTheme?: MatrixFramesByTheme;
};

let cachedMatrixFrames: MatrixFramesByTheme | null = null;
let matrixFramesPromise: Promise<MatrixFramesByTheme> | null = null;
const EMPTY_MATRIX_FRAMES: readonly string[] = [];

const MatrixLoading = ({
  className,
  fit = "stretch",
  height,
  style,
  width,
}: MatrixLoadingProps) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const frameRef = useRef<HTMLDivElement | null>(null);
  const frameIndexRef = useRef(0);
  const { resolvedTheme } = useTheme();
  const matrixColor = useCssVarColor("--foreground");
  const matrixFramesByTheme = useMatrixFrames();
  const matrixFrames =
    matrixFramesByTheme?.[resolvedTheme] ?? EMPTY_MATRIX_FRAMES;
  const resolvedHeight = height ?? style?.height;
  const resolvedWidth = width ?? style?.width;
  const shouldUseResponsiveSize =
    resolvedHeight === undefined && resolvedWidth === undefined;
  const responsiveSize = useResponsiveMatrixSize(
    frameRef,
    shouldUseResponsiveSize,
  );
  const coverCanvasSize = useCoverCanvasSize(frameRef, fit === "cover");
  const canvasStyle =
    fit === "cover"
      ? coverCanvasSize
      : ({
          height: "100%",
          width: "100%",
        } satisfies CSSProperties);

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
      const frame = currentMatrixFrame(matrixFrames, frameIndexRef.current);
      context.clearRect(0, 0, width, height);

      if (frame) {
        drawFrame(context, frame, width, height, matrixColor);
      }
    };

    resizeCanvas();

    const resizeObserver = new ResizeObserver(resizeCanvas);
    resizeObserver.observe(canvas);

    return () => resizeObserver.disconnect();
  }, [matrixColor, matrixFrames]);

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
      const width = Math.max(1, Math.round(rect.width));
      const height = Math.max(1, Math.round(rect.height));
      const frame = currentMatrixFrame(matrixFrames, frameIndexRef.current);

      context.clearRect(0, 0, width, height);

      if (frame) {
        drawFrame(context, frame, width, height, matrixColor);
      }
    };

    if (matrixFrames.length === 0) {
      drawCurrentFrame();
      return;
    }

    if (matrixFrames.length <= 1 || prefersReducedMotion()) {
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
  }, [matrixColor, matrixFrames]);

  return (
    <div
      ref={frameRef}
      className={[
        "box-border overflow-hidden",
        fit === "cover" ? undefined : "relative p-2 sm:p-3",
        className,
      ]
        .filter(Boolean)
        .join(" ")}
      style={{
        ...style,
        aspectRatio:
          style?.aspectRatio ??
          `${matrixRenderBounds.width} / ${matrixRenderBounds.height}`,
        height:
          resolvedHeight ??
          (shouldUseResponsiveSize ? responsiveSize.height : undefined),
        inset: style?.inset ?? (fit === "cover" ? 0 : undefined),
        position: style?.position ?? (fit === "cover" ? "absolute" : undefined),
        width:
          resolvedWidth ??
          (shouldUseResponsiveSize ? responsiveSize.width : "100%"),
      }}
    >
      <canvas
        ref={canvasRef}
        className={
          fit === "cover"
            ? "absolute left-1/2 top-1/2 block -translate-x-1/2 -translate-y-1/2"
            : "block"
        }
        style={canvasStyle}
        role="img"
        aria-label="Skopio loading animation"
      />
    </div>
  );
};

function useMatrixFrames() {
  const [framesByTheme, setFramesByTheme] =
    useState<MatrixFramesByTheme | null>(cachedMatrixFrames);

  useEffect(() => {
    if (cachedMatrixFrames) {
      return;
    }

    let isMounted = true;

    loadMatrixFrames()
      .then((frames) => {
        if (isMounted) {
          setFramesByTheme(frames);
        }
      })
      .catch((error) => {
        console.error("Failed to load matrix loading frames", error);
      });

    return () => {
      isMounted = false;
    };
  }, []);

  return framesByTheme;
}

function loadMatrixFrames() {
  matrixFramesPromise ??= fetch(matrixFramesUrl)
    .then((response) => {
      if (!response.ok) {
        throw new Error(
          `Failed to fetch ${matrixFramesUrl}: ${response.status} ${response.statusText}`,
        );
      }

      return response.json() as Promise<MatrixFramesPayload>;
    })
    .then((payload) => {
      const framesByTheme = payload.framesByTheme;

      if (!framesByTheme?.dark?.length || !framesByTheme?.light?.length) {
        throw new Error(
          "Matrix frame payload is missing light or dark frames.",
        );
      }

      cachedMatrixFrames = framesByTheme;
      return framesByTheme;
    });

  return matrixFramesPromise;
}

function prefersReducedMotion() {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function currentMatrixFrame(frames: readonly string[], frameIndex: number) {
  if (frames.length === 0) {
    return undefined;
  }

  return frames[frameIndex % frames.length];
}

function useCoverCanvasSize(
  ref: RefObject<HTMLDivElement | null>,
  enabled: boolean,
) {
  const [size, setSize] = useState<CSSProperties>({
    height: "100%",
    width: "100%",
  });

  useEffect(() => {
    if (!enabled) {
      return;
    }

    const updateSize = () => {
      const element = ref.current;

      if (!element) {
        return;
      }

      const rect = element.getBoundingClientRect();
      const containerWidth = Math.max(1, rect.width);
      const containerHeight = Math.max(1, rect.height);
      const containerAspectRatio = containerWidth / containerHeight;
      let nextWidth = containerWidth;
      let nextHeight = containerHeight;

      if (containerAspectRatio > MATRIX_ASPECT_RATIO) {
        nextHeight = containerWidth / MATRIX_ASPECT_RATIO;
      } else {
        nextWidth = containerHeight * MATRIX_ASPECT_RATIO;
      }

      setSize((previousSize) => {
        const roundedWidth = `${Math.ceil(nextWidth)}px`;
        const roundedHeight = `${Math.ceil(nextHeight)}px`;

        if (
          previousSize.width === roundedWidth &&
          previousSize.height === roundedHeight
        ) {
          return previousSize;
        }

        return {
          height: roundedHeight,
          width: roundedWidth,
        };
      });
    };

    updateSize();

    const resizeObserver = new ResizeObserver(updateSize);

    if (ref.current) {
      resizeObserver.observe(ref.current);
    }

    window.addEventListener("resize", updateSize);

    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("resize", updateSize);
    };
  }, [enabled, ref]);

  return size;
}

function useResponsiveMatrixSize(
  ref: RefObject<HTMLDivElement | null>,
  enabled: boolean,
) {
  const [size, setSize] = useState<CSSProperties>({
    width: "min(100%, 92vw, 980px)",
  });

  useEffect(() => {
    if (!enabled) {
      return;
    }

    const updateSize = () => {
      const element = ref.current;
      const parentWidth =
        element?.parentElement?.getBoundingClientRect().width ||
        window.innerWidth;
      const viewportWidth = window.innerWidth;
      const viewportHeight = window.innerHeight;
      const availableWidth = Math.min(
        DEFAULT_MAX_WIDTH,
        parentWidth,
        viewportWidth - VIEWPORT_INLINE_GUTTER,
      );
      const availableHeight = Math.max(
        MIN_RESPONSIVE_HEIGHT,
        viewportHeight - VIEWPORT_BLOCK_RESERVE,
      );
      let nextWidth = Math.max(1, availableWidth);
      let nextHeight = nextWidth / MATRIX_ASPECT_RATIO;

      if (nextHeight > availableHeight) {
        nextHeight = availableHeight;
        nextWidth = nextHeight * MATRIX_ASPECT_RATIO;
      }

      setSize((previousSize) => {
        const roundedWidth = `${Math.round(nextWidth)}px`;
        const roundedHeight = `${Math.round(nextHeight)}px`;

        if (
          previousSize.width === roundedWidth &&
          previousSize.height === roundedHeight
        ) {
          return previousSize;
        }

        return {
          height: roundedHeight,
          width: roundedWidth,
        };
      });
    };

    updateSize();

    const resizeObserver = new ResizeObserver(updateSize);
    const observedElement = ref.current?.parentElement ?? ref.current;

    if (observedElement) {
      resizeObserver.observe(observedElement);
    }

    window.addEventListener("resize", updateSize);

    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("resize", updateSize);
    };
  }, [enabled, ref]);

  return size;
}

function drawFrame(
  context: CanvasRenderingContext2D,
  frame: string,
  width: number,
  height: number,
  color: string,
) {
  if (!frame) {
    return;
  }

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
