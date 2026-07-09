import { useElementSize } from "@/hooks/useElementSize";
import { useCssVarColor } from "@/hooks/useChartColor";
import { useTheme } from "@/utils/theme";
import type { CSSProperties } from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
  const {
    ref: canvasRef,
    width: canvasWidth,
    height: canvasHeight,
  } = useElementSize<HTMLCanvasElement>();
  const {
    ref: frameRef,
    width: frameWidth,
    height: frameHeight,
  } = useElementSize<HTMLDivElement>();
  const [frameElement, setFrameElement] = useState<HTMLDivElement | null>(null);
  const { width: parentWidth } = useElementSize<HTMLElement>(
    frameElement?.parentElement ?? null,
  );
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
  const viewportSize = useViewportSize();
  const responsiveSize = useResponsiveMatrixSize(
    shouldUseResponsiveSize,
    parentWidth,
    viewportSize,
  );
  const coverCanvasSize = useCoverCanvasSize(
    fit === "cover",
    frameWidth,
    frameHeight,
  );
  const canvasStyle =
    fit === "cover"
      ? coverCanvasSize
      : ({
          height: "100%",
          width: "100%",
        } satisfies CSSProperties);
  const assignFrameRef = useCallback(
    (node: HTMLDivElement | null) => {
      frameRef.current = node;
      setFrameElement(node);
    },
    [frameRef],
  );

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }

    if (canvasWidth <= 0 || canvasHeight <= 0) {
      return;
    }

    const dpr = window.devicePixelRatio || 1;
    const width = Math.max(1, Math.round(canvasWidth));
    const height = Math.max(1, Math.round(canvasHeight));

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
  }, [canvasHeight, canvasRef, canvasWidth, matrixColor, matrixFrames]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }

    if (canvasWidth <= 0 || canvasHeight <= 0) {
      return;
    }

    const drawCurrentFrame = () => {
      const width = Math.max(1, Math.round(canvasWidth));
      const height = Math.max(1, Math.round(canvasHeight));
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
  }, [canvasHeight, canvasRef, canvasWidth, matrixColor, matrixFrames]);

  return (
    <div
      ref={assignFrameRef}
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

const useMatrixFrames = () => {
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
};

const loadMatrixFrames = () => {
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
};

const prefersReducedMotion = () => {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
};

const currentMatrixFrame = (frames: readonly string[], frameIndex: number) => {
  if (frames.length === 0) {
    return undefined;
  }

  return frames[frameIndex % frames.length];
};

const useCoverCanvasSize = (
  enabled: boolean,
  containerWidth: number,
  containerHeight: number,
) => {
  return useMemo<CSSProperties>(() => {
    if (!enabled) {
      return {
        height: "100%",
        width: "100%",
      };
    }

    if (containerWidth <= 0 || containerHeight <= 0) {
      return {
        height: "100%",
        width: "100%",
      };
    }

    const containerAspectRatio = containerWidth / containerHeight;
    let nextWidth = containerWidth;
    let nextHeight = containerHeight;

    if (containerAspectRatio > MATRIX_ASPECT_RATIO) {
      nextHeight = containerWidth / MATRIX_ASPECT_RATIO;
    } else {
      nextWidth = containerHeight * MATRIX_ASPECT_RATIO;
    }

    return {
      height: `${Math.ceil(nextHeight)}px`,
      width: `${Math.ceil(nextWidth)}px`,
    };
  }, [containerHeight, containerWidth, enabled]);
};

const useResponsiveMatrixSize = (
  enabled: boolean,
  parentWidth: number,
  viewportSize: { width: number; height: number },
) => {
  return useMemo<CSSProperties>(() => {
    if (!enabled) {
      return {
        width: "min(100%, 92vw, 980px)",
      };
    }

    const availableWidth = Math.min(
      DEFAULT_MAX_WIDTH,
      parentWidth || viewportSize.width,
      viewportSize.width - VIEWPORT_INLINE_GUTTER,
    );
    const availableHeight = Math.max(
      MIN_RESPONSIVE_HEIGHT,
      viewportSize.height - VIEWPORT_BLOCK_RESERVE,
    );
    let nextWidth = Math.max(1, availableWidth);
    let nextHeight = nextWidth / MATRIX_ASPECT_RATIO;

    if (nextHeight > availableHeight) {
      nextHeight = availableHeight;
      nextWidth = nextHeight * MATRIX_ASPECT_RATIO;
    }

    return {
      height: `${Math.round(nextHeight)}px`,
      width: `${Math.round(nextWidth)}px`,
    };
  }, [enabled, parentWidth, viewportSize.height, viewportSize.width]);
};

const useViewportSize = () => {
  const [size, setSize] = useState(() => ({
    height: window.innerHeight,
    width: window.innerWidth,
  }));

  useEffect(() => {
    const updateSize = () => {
      setSize({
        height: window.innerHeight,
        width: window.innerWidth,
      });
    };

    window.addEventListener("resize", updateSize);
    return () => {
      window.removeEventListener("resize", updateSize);
    };
  }, []);

  return size;
};

const drawFrame = (
  context: CanvasRenderingContext2D,
  frame: string,
  width: number,
  height: number,
  color: string,
) => {
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
};

const decodeLevel = (charCode: number) => {
  if (charCode >= 48 && charCode <= 57) {
    return charCode - 48;
  }

  if (charCode >= 97 && charCode <= 122) {
    return charCode - 87;
  }

  return 0;
};

export default MatrixLoading;
