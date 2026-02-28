import { useLocalStorage } from "@/hooks/useLocalStorage";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  createInitialObstacles,
  createObstacle,
  dino,
  DinoGameConfig,
  GRAVITY,
  HIGH_SCORE_KEY,
  JUMP_VELOCITY,
  MAX_JUMP_HEIGHT,
  OBSTACLE_SPAWN_GAP,
  ObstacleInstance,
  RUN_FRAME_INTERVAL,
} from "./data";
import { collides, getDinoSprite } from "./frame";

type UseDinoGameOptions = {
  fps: number;
  speed: number;
  stageCols: number;
  stageRows: number;
};

export const useDinoGame = ({
  fps,
  speed,
  stageCols,
  stageRows,
}: UseDinoGameOptions) => {
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

  const config = useMemo<DinoGameConfig>(
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

  useEffect(() => {
    let mounted = true;
    const frameTime = 1000 / fps;

    const animate = (currentTime: number) => {
      if (!mounted) {
        return;
      }

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
          const dinoSprite = getDinoSprite(nextJumpHeight, 0);
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

  return {
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
  };
};
