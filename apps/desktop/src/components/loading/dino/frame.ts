import {
  dino,
  DinoGameConfig,
  dinoLeftUp,
  dinoRightUp,
  Frame,
  ObstacleInstance,
  PIXEL_FONT,
  Sprite,
} from "./data";

const createEmptyFrame = (rows: number, cols: number) =>
  Array.from({ length: rows }, () => Array(cols).fill(0));

export const drawSprite = (
  frame: Frame,
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

export const drawPixelText = (
  frame: Frame,
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

export const getPixelTextWidth = (text: string) =>
  [...text].reduce(
    (width, char) =>
      width + (PIXEL_FONT[char] ?? PIXEL_FONT[" "])[0].length + 1,
    -1,
  );

export const collides = (
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

export const getDinoSprite = (jumpHeight: number, runFrame: number) => {
  if (jumpHeight > 0) {
    return dino;
  }

  return runFrame === 0 ? dinoLeftUp : dinoRightUp;
};

type BuildDinoFrameOptions = {
  config: DinoGameConfig;
  highScore: number;
  isGameOver: boolean;
  jumpHeight: number;
  obstacleState: ObstacleInstance[];
  runFrame: number;
  score: number;
  stageCols: number;
  stageRows: number;
};

export const buildDinoFrame = ({
  config,
  highScore,
  isGameOver,
  jumpHeight,
  obstacleState,
  runFrame,
  score,
  stageCols,
  stageRows,
}: BuildDinoFrameOptions) => {
  const frame = createEmptyFrame(stageRows, stageCols);
  const hudText = `HI ${highScore} ${score}`;

  for (let c = 0; c < stageCols; c++) {
    frame[stageRows - 1][c] = 1;
  }

  drawPixelText(frame, hudText, stageCols - getPixelTextWidth(hudText) - 2, 2);

  const dinoY = Math.round(config.dinoBaseY - jumpHeight);
  const dinoSprite = getDinoSprite(jumpHeight, runFrame);

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
};
