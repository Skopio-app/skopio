import { useCssVarColor } from "@/hooks/useChartColor";
import { useLocalStorage } from "@/hooks/useLocalStorage";
import { useWindowKeydown } from "@/hooks/useWindowKeydown";
import { useEffect, useMemo, useRef, useState } from "react";

const dino = [
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0],
  [1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
  [1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
  [1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
  [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0],
  [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
  [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
  [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

const dinoLeftUp = [
  ...dino.slice(0, 18),
  [0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

const dinoRightUp = [
  ...dino.slice(0, 18),
  [0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

const obstacles = [
  [
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0],
    [0, 1, 0, 1, 0, 0, 0],
    [0, 1, 1, 1, 0, 1, 0],
    [0, 0, 0, 1, 1, 1, 0],
    [0, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0],
  ],
  [
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 1, 0, 1, 0],
    [0, 1, 0, 1, 1, 1, 0],
    [0, 1, 1, 1, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0],
  ],
  [
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0],
    [0, 1, 0, 1, 0, 1, 0],
    [0, 1, 1, 1, 1, 1, 0],
    [0, 0, 0, 1, 0, 0, 0],
    [0, 0, 0, 1, 0, 0, 0],
  ],
];

type Sprite = number[][];

type ObstacleInstance = {
  id: number;
  sprite: Sprite;
  x: number;
  scored: boolean;
};

type DinoLoadingProps = {
  fps?: number;
  size?: number;
  gap?: number;
  stageCols?: number;
  stageRows?: number;
  speed?: number;
};

const HIGH_SCORE_KEY = "dino-loading-high-score";
const INITIAL_OBSTACLE_OFFSET = 18;
const OBSTACLE_SPAWN_GAP = 56;
const JUMP_VELOCITY = 5.8;
const GRAVITY = 0.9;
const MAX_JUMP_HEIGHT = 13;
const RUN_FRAME_INTERVAL = 4;
const PIXEL_FONT: Record<string, number[][]> = {
  "0": [
    [1, 1, 1],
    [1, 0, 1],
    [1, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
  ],
  "1": [
    [0, 1, 0],
    [1, 1, 0],
    [0, 1, 0],
    [0, 1, 0],
    [1, 1, 1],
  ],
  "2": [
    [1, 1, 1],
    [0, 0, 1],
    [1, 1, 1],
    [1, 0, 0],
    [1, 1, 1],
  ],
  "3": [
    [1, 1, 1],
    [0, 0, 1],
    [0, 1, 1],
    [0, 0, 1],
    [1, 1, 1],
  ],
  "4": [
    [1, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
    [0, 0, 1],
    [0, 0, 1],
  ],
  "5": [
    [1, 1, 1],
    [1, 0, 0],
    [1, 1, 1],
    [0, 0, 1],
    [1, 1, 1],
  ],
  "6": [
    [1, 1, 1],
    [1, 0, 0],
    [1, 1, 1],
    [1, 0, 1],
    [1, 1, 1],
  ],
  "7": [
    [1, 1, 1],
    [0, 0, 1],
    [0, 1, 0],
    [0, 1, 0],
    [0, 1, 0],
  ],
  "8": [
    [1, 1, 1],
    [1, 0, 1],
    [1, 1, 1],
    [1, 0, 1],
    [1, 1, 1],
  ],
  "9": [
    [1, 1, 1],
    [1, 0, 1],
    [1, 1, 1],
    [0, 0, 1],
    [1, 1, 1],
  ],
  A: [
    [0, 1, 0],
    [1, 0, 1],
    [1, 1, 1],
    [1, 0, 1],
    [1, 0, 1],
  ],
  E: [
    [1, 1, 1],
    [1, 0, 0],
    [1, 1, 0],
    [1, 0, 0],
    [1, 1, 1],
  ],
  G: [
    [0, 1, 1],
    [1, 0, 0],
    [1, 0, 1],
    [1, 0, 1],
    [0, 1, 1],
  ],
  H: [
    [1, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
    [1, 0, 1],
    [1, 0, 1],
  ],
  I: [
    [1, 1, 1],
    [0, 1, 0],
    [0, 1, 0],
    [0, 1, 0],
    [1, 1, 1],
  ],
  M: [
    [1, 0, 1],
    [1, 1, 1],
    [1, 1, 1],
    [1, 0, 1],
    [1, 0, 1],
  ],
  O: [
    [1, 1, 1],
    [1, 0, 1],
    [1, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
  ],
  R: [
    [1, 1, 0],
    [1, 0, 1],
    [1, 1, 0],
    [1, 0, 1],
    [1, 0, 1],
  ],
  S: [
    [1, 1, 1],
    [1, 0, 0],
    [1, 1, 1],
    [0, 0, 1],
    [1, 1, 1],
  ],
  B: [
    [1, 1, 0],
    [1, 0, 1],
    [1, 1, 0],
    [1, 0, 1],
    [1, 1, 0],
  ],
  V: [
    [1, 0, 1],
    [1, 0, 1],
    [1, 0, 1],
    [1, 0, 1],
    [0, 1, 0],
  ],
  ":": [[0], [1], [0], [1], [0]],
  " ": [[0], [0], [0], [0], [0]],
};

const createObstacle = (
  stageCols: number,
  id: number,
  lastObstacle?: ObstacleInstance,
): ObstacleInstance => {
  const sprite = obstacles[Math.floor(Math.random() * obstacles.length)];
  const x = lastObstacle
    ? lastObstacle.x + OBSTACLE_SPAWN_GAP + Math.floor(Math.random() * 18)
    : stageCols + INITIAL_OBSTACLE_OFFSET;

  return {
    id,
    sprite,
    x,
    scored: false,
  };
};

const createInitialObstacles = (stageCols: number) => {
  const first = createObstacle(stageCols, 0);
  const second = createObstacle(stageCols, 1, first);
  return {
    nextId: 2,
    obstacles: [first, second],
  };
};

const createEmptyFrame = (rows: number, cols: number) =>
  Array.from({ length: rows }, () => Array(cols).fill(0));

const drawSprite = (
  frame: number[][],
  sprite: Sprite,
  startX: number,
  startY: number,
) => {
  for (let r = 0; r < sprite.length; r++) {
    for (let c = 0; c < sprite[0].length; c++) {
      const drawX = startX + c;
      const drawY = startY + r;
      if (
        drawX >= 0 &&
        drawX < frame[0].length &&
        drawY >= 0 &&
        drawY < frame.length &&
        sprite[r][c]
      ) {
        frame[drawY][drawX] = 1;
      }
    }
  }
};

const drawPixelText = (
  frame: number[][],
  text: string,
  startX: number,
  startY: number,
) => {
  let offsetX = startX;

  for (const char of text) {
    const sprite = PIXEL_FONT[char] ?? PIXEL_FONT[" "];
    drawSprite(frame, sprite, offsetX, startY);
    offsetX += sprite[0].length + 1;
  }
};

const getPixelTextWidth = (text: string) =>
  [...text].reduce(
    (width, char) =>
      width + (PIXEL_FONT[char] ?? PIXEL_FONT[" "])[0].length + 1,
    -1,
  );

const collides = (
  dinoSprite: Sprite,
  dinoX: number,
  dinoY: number,
  obstacleSprite: Sprite,
  obstacleX: number,
  obstacleY: number,
) => {
  const left = Math.max(dinoX, obstacleX);
  const right = Math.min(
    dinoX + dinoSprite[0].length,
    obstacleX + obstacleSprite[0].length,
  );
  const top = Math.max(dinoY, obstacleY);
  const bottom = Math.min(
    dinoY + dinoSprite.length,
    obstacleY + obstacleSprite.length,
  );

  if (left >= right || top >= bottom) {
    return false;
  }

  for (let y = top; y < bottom; y++) {
    for (let x = left; x < right; x++) {
      if (
        dinoSprite[y - dinoY][x - dinoX] &&
        obstacleSprite[y - obstacleY][x - obstacleX]
      ) {
        return true;
      }
    }
  }

  return false;
};

const DinoLoading = ({
  fps = 24,
  size = 2,
  gap = 1,
  stageCols = 160,
  stageRows = 76,
  speed = 2,
}: DinoLoadingProps) => {
  const initialObstacles = useMemo(
    () => createInitialObstacles(stageCols),
    [stageCols],
  );
  const [jumpHeight, setJumpHeight] = useState(0);
  const [score, setScore] = useState(0);
  const [highScore, setHighScore] = useLocalStorage<number>(HIGH_SCORE_KEY, 0);
  const [isGameOver, setIsGameOver] = useState(false);
  const [isStarted, setIsStarted] = useState(false);
  const [runFrame, setRunFrame] = useState(0);
  const [obstacleState, setObstacleState] = useState<ObstacleInstance[]>(
    initialObstacles.obstacles,
  );
  const animationRef = useRef<number | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const lastTimeRef = useRef(0);
  const velocityRef = useRef(0);
  const obstacleIdRef = useRef(initialObstacles.nextId);
  const jumpHeightRef = useRef(0);
  const scoreRef = useRef(0);
  const startedRef = useRef(false);
  const gameOverRef = useRef(false);
  const obstacleStateRef = useRef<ObstacleInstance[]>(
    initialObstacles.obstacles,
  );
  const runStepRef = useRef(0);

  const config = useMemo(
    () => ({
      dinoX: 5,
      dinoBaseY: stageRows - 1 - dino.length,
    }),
    [stageRows],
  );

  useEffect(() => {
    jumpHeightRef.current = jumpHeight;
  }, [jumpHeight]);

  useEffect(() => {
    scoreRef.current = score;
  }, [score]);

  useEffect(() => {
    startedRef.current = isStarted;
  }, [isStarted]);

  useEffect(() => {
    gameOverRef.current = isGameOver;
  }, [isGameOver]);

  useEffect(() => {
    obstacleStateRef.current = obstacleState;
  }, [obstacleState]);

  const resetGame = () => {
    const nextInitialObstacles = createInitialObstacles(stageCols);
    velocityRef.current = 0;
    jumpHeightRef.current = 0;
    scoreRef.current = 0;
    startedRef.current = false;
    gameOverRef.current = false;
    runStepRef.current = 0;
    obstacleIdRef.current = nextInitialObstacles.nextId;
    obstacleStateRef.current = nextInitialObstacles.obstacles;
    setJumpHeight(0);
    setScore(0);
    setIsGameOver(false);
    setIsStarted(false);
    setRunFrame(0);
    setObstacleState(nextInitialObstacles.obstacles);
  };

  const jump = () => {
    if (gameOverRef.current) {
      resetGame();
      setIsStarted(true);
      startedRef.current = true;
      velocityRef.current = JUMP_VELOCITY;
      return;
    }

    if (jumpHeightRef.current > 0) {
      return;
    }

    setIsStarted(true);
    startedRef.current = true;
    velocityRef.current = JUMP_VELOCITY;
  };

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

  useEffect(() => {
    let mounted = true;
    const frameTime = 1000 / fps;

    const animate = (currentTime: number) => {
      if (!mounted) return;

      if (currentTime - lastTimeRef.current >= frameTime) {
        if (startedRef.current && !gameOverRef.current) {
          velocityRef.current -= GRAVITY;
          const nextJumpHeight = Math.min(
            MAX_JUMP_HEIGHT,
            Math.max(0, jumpHeightRef.current + velocityRef.current),
          );
          jumpHeightRef.current = nextJumpHeight;
          setJumpHeight(nextJumpHeight);
          if (nextJumpHeight >= MAX_JUMP_HEIGHT && velocityRef.current > 0) {
            velocityRef.current = 0;
          }
          if (nextJumpHeight === 0 && velocityRef.current < 0) {
            velocityRef.current = 0;
          }
          if (nextJumpHeight === 0) {
            runStepRef.current += 1;
            if (runStepRef.current >= RUN_FRAME_INTERVAL) {
              runStepRef.current = 0;
              setRunFrame((prev) => (prev + 1) % 2);
            }
          }

          const shifted = obstacleStateRef.current
            .map((obstacle) => ({ ...obstacle, x: obstacle.x - speed }))
            .filter((obstacle) => obstacle.x + obstacle.sprite[0].length > -2);

          let nextScore = scoreRef.current;
          const scoredObstacles = shifted.map((obstacle) => {
            if (
              !obstacle.scored &&
              obstacle.x + obstacle.sprite[0].length < config.dinoX
            ) {
              nextScore += 1;
              return { ...obstacle, scored: true };
            }

            return obstacle;
          });

          const lastObstacle = scoredObstacles[scoredObstacles.length - 1];
          if (
            !lastObstacle ||
            lastObstacle.x < stageCols - OBSTACLE_SPAWN_GAP
          ) {
            scoredObstacles.push(
              createObstacle(stageCols, obstacleIdRef.current, lastObstacle),
            );
            obstacleIdRef.current += 1;
          }

          const dinoY = Math.round(config.dinoBaseY - nextJumpHeight);
          const dinoSprite = nextJumpHeight > 0 ? dino : dinoLeftUp;
          const hitObstacle = scoredObstacles.some((obstacle) =>
            collides(
              dinoSprite,
              config.dinoX,
              dinoY,
              obstacle.sprite,
              Math.round(obstacle.x),
              stageRows - 1 - obstacle.sprite.length,
            ),
          );

          obstacleStateRef.current = scoredObstacles;
          setObstacleState(scoredObstacles);

          if (nextScore !== scoreRef.current) {
            scoreRef.current = nextScore;
            setScore(nextScore);
            setHighScore((prevScore) => Math.max(prevScore, nextScore));
          }

          if (hitObstacle) {
            gameOverRef.current = true;
            startedRef.current = false;
            setIsGameOver(true);
            setIsStarted(false);
            setHighScore((prevScore) => Math.max(prevScore, nextScore));
          }
        }

        lastTimeRef.current = currentTime;
      }

      animationRef.current = requestAnimationFrame(animate);
    };

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      mounted = false;
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [
    config.dinoBaseY,
    config.dinoX,
    fps,
    setHighScore,
    speed,
    stageCols,
    stageRows,
  ]);

  const currentFrame = useMemo(() => {
    const frame = createEmptyFrame(stageRows, stageCols);
    const hudText = `HI ${highScore} ${score}`;

    for (let c = 0; c < stageCols; c++) {
      frame[stageRows - 1][c] = 1;
    }

    const hudTextWidth = getPixelTextWidth(hudText);
    drawPixelText(frame, hudText, stageCols - hudTextWidth - 2, 2);

    const dinoY = Math.round(config.dinoBaseY - jumpHeight);
    const dinoSprite =
      jumpHeight > 0 ? dino : runFrame === 0 ? dinoLeftUp : dinoRightUp;
    obstacleState.forEach((obstacle) => {
      drawSprite(
        frame,
        obstacle.sprite,
        Math.round(obstacle.x),
        stageRows - 1 - obstacle.sprite.length,
      );
    });

    drawSprite(frame, dinoSprite, config.dinoX, dinoY);

    if (isGameOver) {
      const gameOverText = "GAME OVER";
      const gameOverWidth = getPixelTextWidth(gameOverText);
      const textX = Math.floor((stageCols - gameOverWidth) / 2);
      const textY = Math.max(9, Math.floor(stageRows / 2) - 2);
      drawPixelText(frame, gameOverText, textX, textY);
    }

    return frame;
  }, [
    config.dinoBaseY,
    config.dinoX,
    highScore,
    isGameOver,
    jumpHeight,
    obstacleState,
    runFrame,
    score,
    stageCols,
    stageRows,
  ]);

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
