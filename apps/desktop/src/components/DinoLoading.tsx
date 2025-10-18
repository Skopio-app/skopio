import { Matrix, type Frame } from "@skopio/ui";
import { useMemo } from "react";

const dino: Frame = [
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

const dinoLeftUp: Frame = [
  ...dino.slice(0, 18),
  [0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

const dinoRightUp: Frame = [
  ...dino.slice(0, 18),
  [0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

const cactus1: Frame = [
  [0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 1, 0, 0, 0],
  [0, 1, 0, 1, 0, 0, 0],
  [0, 1, 1, 1, 0, 1, 0],
  [0, 0, 0, 1, 1, 1, 0],
  [0, 0, 0, 1, 0, 0, 0],
  [0, 0, 0, 1, 0, 0, 0],
];

const cactus2: Frame = [
  [0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 1, 0, 0, 0],
  [0, 0, 0, 1, 0, 1, 0],
  [0, 1, 0, 1, 1, 1, 0],
  [0, 1, 1, 1, 0, 0, 0],
  [0, 0, 0, 1, 0, 0, 0],
  [0, 0, 0, 1, 0, 0, 0],
];

const cactus3: Frame = [
  [0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 1, 0, 0, 0],
  [0, 1, 0, 1, 0, 1, 0],
  [0, 1, 1, 1, 1, 1, 0],
  [0, 0, 0, 1, 0, 0, 0],
  [0, 0, 0, 1, 0, 0, 0],
];

const cactus4: Frame = [
  [0, 0, 0, 0, 0, 0, 0],
  [0, 1, 0, 1, 0, 1, 0],
  [0, 1, 0, 1, 0, 1, 0],
  [0, 1, 1, 1, 1, 1, 0],
  [1, 0, 0, 1, 0, 0, 1],
  [1, 1, 1, 1, 1, 1, 1],
  [0, 0, 0, 1, 0, 0, 0],
];

const skeleton: Frame = [
  [0, 0, 1, 1, 1, 0, 0],
  [0, 0, 1, 1, 1, 0, 0],
  [1, 0, 0, 1, 0, 0, 1],
  [1, 1, 1, 1, 1, 1, 1],
  [0, 0, 0, 1, 0, 0, 0],
  [0, 0, 1, 0, 1, 0, 0],
  [1, 1, 1, 0, 1, 1, 1],
];

const groundFrames: number[][] = [
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  [1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1],
];

const BLANK = (rows: number, cols: number): Frame =>
  Array.from({ length: rows }, () => Array(cols).fill(0));

const blit = (stage: Frame, sprite: Frame, x: number, y: number) => {
  const H = stage.length,
    W = stage[0].length;
  const h = sprite.length,
    w = sprite[0].length;
  for (let r = 0; r < h; r++) {
    const sr = y + r;
    if (sr < 0 || sr >= H) continue;
    for (let c = 0; c < w; c++) {
      const sc = x + c;
      if (sc < 0 || sc >= W) continue;
      if (sprite[r][c]) stage[sr][sc] = 1;
    }
  }
};

const drawGround = (stage: Frame, t: number, groundH: number) => {
  const H = stage.length;
  const W = stage[0].length;
  const bottom = H - 1;

  for (let c = 0; c < W; c++) stage[bottom][c] = 1;

  for (let g = 1; g < groundH; g++) {
    const row = bottom - g;
    const pat = groundFrames[g % groundFrames.length];
    for (let c = 0; c < W; c++) {
      const pebble = pat[(c + t + g * 3) % pat.length];
      stage[row][c] = pebble;
    }
  }
};

const makeArc = ({
  maxHeight = 10,
  up = 6, // frames going up
  hold = 2, // frames at peak
  down = 6, // frames going down
}: {
  maxHeight?: number;
  up?: number;
  hold?: number;
  down?: number;
}) => {
  const rise = Array.from({ length: up }, (_, i) =>
    Math.round((i + 1) * (maxHeight / up)),
  );
  const peak = Array(hold).fill(maxHeight);
  const fall = Array.from({ length: down }, (_, i) =>
    Math.round(maxHeight - (i + 1) * (maxHeight / down)),
  ).map((v) => Math.max(0, v));
  return [...rise, ...peak, ...fall];
};

const buildAnimation = ({
  stageCols = 120,
  groundH = 2,
  skyHeadroom = 10,
  repeats = 4,
  scrollPerFrame = 3, // columns moved per frame (speed control)
}: {
  stageCols?: number;
  groundH?: number;
  skyHeadroom?: number;
  repeats?: number;
  scrollPerFrame?: number;
} = {}): Frame[] => {
  const dinoH = dino.length; // 21
  const rows = dinoH + groundH + skyHeadroom;
  const cols = stageCols;

  const dinoX = 5;
  const dinoFrontFoot = dinoX + 12; // measure timing from the toe
  const dinoBaseY = rows - groundH - dinoH;

  const obstacleSeq = [cactus1, cactus3, skeleton, cactus2, cactus4];
  const gap = Math.floor(stageCols / 3); // spacing between cacti
  const firstStart = cols + 6;

  // Repeat cacti across the track
  const allObstacles: Frame[] = Array.from({ length: repeats }).flatMap(
    () => obstacleSeq,
  );
  const obstacleStarts = allObstacles.map((_, i) => firstStart + i * gap);

  const lastStart = obstacleStarts[obstacleStarts.length - 1];
  const totalWorld = lastStart + cols + 12;

  // Taller arc
  const jumpArc = makeArc({ maxHeight: 9, up: 6, hold: 2, down: 6 });
  const L = jumpArc.length;

  // Start jump when cactus *just* clips the toe
  const overlapStart = 1;
  const startDist = -overlapStart;

  const frames: Frame[] = [];
  for (let worldX = 0; worldX < totalWorld; worldX += scrollPerFrame) {
    const frame = BLANK(rows, cols);
    const t = Math.floor(worldX / scrollPerFrame);

    drawGround(frame, t, groundH);

    // Scroll cacti by worldX
    allObstacles.forEach((cx, i) => {
      const x = obstacleStarts[i] - worldX;
      const y = rows - groundH - cx.length;
      blit(frame, cx, x, y);
    });

    // Jump control: find the nearest cactus intersecting our trigger window
    let jumpOffset = 0;
    for (let i = 0; i < allObstacles.length; i++) {
      const cactusLeadX = obstacleStarts[i] - worldX; // left edge of cactus
      const distLead = cactusLeadX - dinoFrontFoot; // from toe to cactus edge
      // Index along arc: 1 column → 1 arc step
      const idx = startDist - distLead;
      if (idx >= 0 && idx < L) {
        jumpOffset = jumpArc[idx];
        break;
      }
    }

    // Gait
    const run = t % 2 === 0 ? dinoLeftUp : dinoRightUp;
    blit(frame, run, dinoX, dinoBaseY - jumpOffset);
    frames.push(frame);
  }

  return frames;
};

const DinoLoading = ({
  fps = 24,
  size = 6,
  gap = 2,
  stageCols = 120,
  speed = 5,
}: {
  fps?: number;
  size?: number;
  gap?: number;
  stageCols?: number;
  speed?: number;
}) => {
  const frames = useMemo(
    () =>
      buildAnimation({
        stageCols,
        groundH: 2,
        skyHeadroom: 10,
        repeats: 4,
        scrollPerFrame: speed,
      }),
    [stageCols, speed],
  );

  return (
    <div className="inline-flex items-center gap-3 rounded-md bg-muted/20 px-3 py-2">
      <Matrix
        rows={frames[0].length}
        cols={frames[0][0].length}
        frames={frames}
        fps={fps}
        loop
        autoplay
        size={size}
        gap={gap}
        palette={{
          on: "hsl(0 0% 12%)",
          off: "hsl(0 0% 88%)",
        }}
        ariaLabel="Long path dino runner"
        className="shrink-0"
      />
    </div>
  );
};

export default DinoLoading;
