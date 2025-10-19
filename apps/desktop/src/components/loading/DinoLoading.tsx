import { useCssVarColor } from "@/hooks/useChartColor";
import { useState, useEffect, useRef, useMemo } from "react";

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
  [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 18
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
  [
    [0, 0, 1, 1, 1, 0, 0],
    [0, 0, 1, 1, 1, 0, 0],
    [1, 0, 0, 1, 0, 0, 1],
    [1, 1, 1, 1, 1, 1, 1],
    [0, 0, 0, 1, 0, 0, 0],
    [0, 0, 1, 0, 1, 0, 0],
    [1, 1, 1, 0, 1, 1, 1],
  ],
];

const DinoLoading = ({
  fps = 24,
  size = 6,
  gap = 2,
  stageCols = 100,
  stageRows = 36,
  speed = 2,
}) => {
  const [worldX, setWorldX] = useState(0);
  const animationRef = useRef<number | null>(null);
  const lastTimeRef = useRef(0);

  const config = useMemo(
    () => ({
      dinoX: 5,
      dinoBaseY: stageRows - 1 - dino.length,
      obstacleGap: Math.floor(stageCols / 2.5),
      jumpArc: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
    }),
    [stageCols, stageRows],
  );

  // Calculate total animation length
  const totalLength = useMemo(() => {
    const repeats = 3;
    const lastObstacleX =
      stageCols + config.obstacleGap * (repeats * obstacles.length - 1);
    return lastObstacleX + stageCols; // Extra space to clear the screen
  }, [stageCols, config.obstacleGap]);

  useEffect(() => {
    let mounted = true;
    const frameTime = 1000 / fps;

    const animate = (currentTime: number) => {
      if (!mounted) return;

      if (currentTime - lastTimeRef.current >= frameTime) {
        setWorldX((prev) => {
          const next = prev + speed;
          // Loop back to start when we've cleared all obstacles
          return next >= totalLength ? 0 : next;
        });
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
  }, [fps, speed, totalLength]);

  // Generate current frame on-demand
  const currentFrame = useMemo(() => {
    const frame = Array.from({ length: stageRows }, () =>
      Array(stageCols).fill(0),
    );

    // Ground line only (no pebbles)
    for (let c = 0; c < stageCols; c++) {
      frame[stageRows - 1][c] = 1;
    }

    // Obstacles (cycle through 3 times)
    for (let rep = 0; rep < 3; rep++) {
      obstacles.forEach((obs, i) => {
        const x =
          stageCols +
          config.obstacleGap * (rep * obstacles.length + i) -
          worldX;
        if (x > -obs[0].length && x < stageCols) {
          const y = stageRows - 1 - obs.length;
          for (let r = 0; r < obs.length; r++) {
            for (let c = 0; c < obs[0].length; c++) {
              const drawX = x + c;
              const drawY = y + r;
              if (drawX >= 0 && drawX < stageCols && drawY >= 0 && obs[r][c]) {
                frame[drawY][drawX] = 1;
              }
            }
          }
        }
      });
    }

    // Dino with jump logic
    let jumpOffset = 0;
    const dinoFrontX = config.dinoX + 15;
    for (let rep = 0; rep < 3; rep++) {
      obstacles.forEach((_obs, i) => {
        const obsX =
          stageCols +
          config.obstacleGap * (rep * obstacles.length + i) -
          worldX;
        const dist = obsX - dinoFrontX;
        const idx = Math.round(-dist);
        if (idx >= 0 && idx < config.jumpArc.length) {
          jumpOffset = config.jumpArc[idx];
        }
      });
    }

    // Use static dino when jumping, alternate legs when on ground
    const dinoSprite =
      jumpOffset > 0
        ? dino
        : Math.floor(worldX / 4) % 2 === 0
          ? dinoLeftUp
          : dinoRightUp;
    const dinoY = config.dinoBaseY - jumpOffset;
    for (let r = 0; r < dinoSprite.length; r++) {
      for (let c = 0; c < dinoSprite[0].length; c++) {
        const drawX = config.dinoX + c;
        const drawY = dinoY + r;
        if (
          drawX >= 0 &&
          drawX < stageCols &&
          drawY >= 0 &&
          drawY < stageRows &&
          dinoSprite[r][c]
        ) {
          frame[drawY][drawX] = 1;
        }
      }
    }

    return frame;
  }, [worldX, stageCols, stageRows, config]);

  // Simple SVG renderer
  const cellSize = size;
  const cellGap = gap;
  const width = stageCols * (cellSize + cellGap) - cellGap;
  const height = stageRows * (cellSize + cellGap) - cellGap;

  const on = useCssVarColor("--foreground");
  const off = useCssVarColor("--muted");

  return (
    <div className="inline-flex items-center gap-3 rounded-md px-3 py-2">
      <svg width={width} height={height} role="img" aria-label="Dino runner">
        {currentFrame.map((row, r) =>
          row.map((cell, c) => (
            <circle
              key={`${r}-${c}`}
              cx={c * (cellSize + cellGap) + cellSize / 2}
              cy={r * (cellSize + cellGap) + cellSize / 2}
              r={cellSize / 2}
              fill={cell ? on : off}
            />
          )),
        )}
      </svg>
    </div>
  );
};

export default DinoLoading;
