import { useCssVarColor } from "@/hooks/useChartColor";
import { useWindowKeydown } from "@/hooks/useWindowKeydown";
import { useEffect, useMemo, useRef } from "react";
import { buildDinoFrame } from "./dino/frame";
import { useDinoGame } from "./dino/useDinoGame";

type DinoLoadingProps = {
  fps?: number;
  size?: number;
  gap?: number;
  stageCols?: number;
  stageRows?: number;
  speed?: number;
};

const DinoLoading = ({
  fps = 24,
  size = 2,
  gap = 1,
  stageCols = 160,
  stageRows = 76,
  speed = 2,
}: DinoLoadingProps) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const {
    config,
    highScore,
    isGameOver,
    isStarted,
    jump,
    jumpHeight,
    obstacleState,
    resetGame,
    runFrame,
    score,
  } = useDinoGame({
    fps,
    speed,
    stageCols,
    stageRows,
  });

  useWindowKeydown((event) => {
    if (
      event.code === "Space" ||
      event.code === "ArrowUp" ||
      event.code === "KeyW"
    ) {
      event.preventDefault();
      jump();
    }

    if (event.code === "KeyR") {
      event.preventDefault();
      resetGame();
    }
  });

  const currentFrame = useMemo(
    () =>
      buildDinoFrame({
        config,
        highScore,
        isGameOver,
        jumpHeight,
        obstacleState,
        runFrame,
        score,
        stageCols,
        stageRows,
      }),
    [
      config,
      highScore,
      isGameOver,
      jumpHeight,
      obstacleState,
      runFrame,
      score,
      stageCols,
      stageRows,
    ],
  );

  const cellSize = size;
  const cellGap = gap;
  const width = stageCols * (cellSize + cellGap) - cellGap;
  const height = stageRows * (cellSize + cellGap) - cellGap;

  const on = useCssVarColor("--foreground");
  const off = useCssVarColor("--muted");

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }

    const dpr = window.devicePixelRatio || 1;
    const displayWidth = Math.max(1, Math.round(width));
    const displayHeight = Math.max(1, Math.round(height));

    if (
      canvas.width !== displayWidth * dpr ||
      canvas.height !== displayHeight * dpr
    ) {
      canvas.width = displayWidth * dpr;
      canvas.height = displayHeight * dpr;
      canvas.style.width = `${displayWidth}px`;
      canvas.style.height = `${displayHeight}px`;
    }

    context.setTransform(dpr, 0, 0, dpr, 0, 0);
    context.clearRect(0, 0, displayWidth, displayHeight);

    for (let r = 0; r < currentFrame.length; r++) {
      for (let c = 0; c < currentFrame[r].length; c++) {
        context.fillStyle = currentFrame[r][c] ? on : off;
        context.fillRect(
          c * (cellSize + cellGap),
          r * (cellSize + cellGap),
          cellSize,
          cellSize,
        );
      }
    }
  }, [cellGap, cellSize, currentFrame, height, off, on, width]);

  return (
    <div className="inline-flex flex-col items-center gap-3 rounded-md px-3 py-2">
      <button
        type="button"
        onClick={jump}
        className="group relative rounded-md border border-border/60 bg-transparent p-0 text-left outline-none transition-colors hover:border-foreground/40 focus-visible:border-foreground/70"
        aria-label="Play dino runner"
      >
        <canvas
          ref={canvasRef}
          width={width}
          height={height}
          role="img"
          aria-label="Dino runner"
        />
      </button>
      <div className="flex flex-col items-center text-xs text-muted-foreground">
        <p>
          {isGameOver
            ? "Crash. Press Space or click to restart."
            : "Press Space, Up, W, or click to jump."}
        </p>
        <p>
          {isStarted
            ? "Clear each cactus to increase your score."
            : "The game starts on your first jump."}
        </p>
      </div>
    </div>
  );
};

export default DinoLoading;
