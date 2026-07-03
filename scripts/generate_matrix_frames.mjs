#!/usr/bin/env node

import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(SCRIPT_DIR, "..");
const DEFAULT_OUTPUT = path.join(
  REPO_ROOT,
  "apps/desktop/src/components/loading/matrix/generatedFrames.ts",
);
const DEFAULT_DATA_OUTPUT = path.join(
  REPO_ROOT,
  "apps/desktop/public/loading/matrixFrames.json",
);
const ENCODING = "0123456789abcdefghijklmnopqrstuvwxyz";

const options = parseArgs(process.argv.slice(2));

if (!options.input) {
  printUsage();
  process.exit(1);
}

const maxColumns = positiveInteger(options.columns, 120, "columns");
const maxRows = positiveInteger(options.rows, 64, "rows");
const fps = positiveInteger(options.fps, 12, "fps");
const maxFrames = positiveInteger(options.frames, 180, "frames");
const levels = positiveInteger(options.levels, 12, "levels");
const contrast = positiveNumber(options.contrast, 1, "contrast");
const gamma = positiveNumber(options.gamma, 1, "gamma");
const brightness = numberOption(options.brightness, 0, "brightness");
const blackPoint = numberOption(options.black_point, 0, "black-point");
const whitePoint = numberOption(options.white_point, 255, "white-point");
const autoLevels = options.auto_levels === true;
const minLevel = numberOption(options.min_level, 0, "min-level");
const threshold =
  options.threshold === undefined
    ? undefined
    : numberOption(options.threshold, 0, "threshold");
const invert = options.invert === true;
const output = options.output ?? DEFAULT_OUTPUT;
const dataOutput = options.data_output ?? DEFAULT_DATA_OUTPUT;

if (levels < 2 || levels > ENCODING.length) {
  throw new Error(`--levels must be between 2 and ${ENCODING.length}`);
}

if (blackPoint >= whitePoint) {
  throw new Error("--black-point must be lower than --white-point");
}

if (minLevel < 0 || minLevel > 1) {
  throw new Error("--min-level must be between 0 and 1");
}

if (threshold !== undefined && (threshold < 0 || threshold > 1)) {
  throw new Error("--threshold must be between 0 and 1");
}

const mediaSize = await readMediaSize(options.input);
const outputSize = containInGrid({
  sourceWidth: mediaSize.width,
  sourceHeight: mediaSize.height,
  columns: maxColumns,
  rows: maxRows,
});
const columns = maxColumns;
const rows = maxRows;
const frameSize = columns * rows;
const rawFrames = await readRawGrayFrames({
  input: options.input,
  columns,
  rows,
  outputSize,
  fps,
  maxFrames,
});

if (rawFrames.length === 0) {
  throw new Error(
    "No frames were generated. Check the input path and ffmpeg output.",
  );
}

const mapping = {
  autoLevels,
  blackPoint,
  brightness,
  contrast,
  gamma,
  invert,
  levels,
  minLevel,
  threshold,
  whitePoint,
};
const themeMappings = {
  dark: mapping,
  light: {
    ...mapping,
    invert: !mapping.invert,
    minLevel: 0,
  },
};
const renderBounds = {
  x: Math.floor((columns - outputSize.width) / 2),
  y: Math.floor((rows - outputSize.height) / 2),
  width: outputSize.width,
  height: outputSize.height,
};
const matrixFramesByTheme = {
  dark: rawFrames.map((frame) =>
    frameToMatrixString(
      frame,
      columns,
      levels,
      themeMappings.dark,
      renderBounds,
    ),
  ),
  light: rawFrames.map((frame) =>
    frameToMatrixString(
      frame,
      columns,
      levels,
      themeMappings.light,
      renderBounds,
    ),
  ),
};

await writeFrameModule({
  output,
  dataOutput,
  source: options.input,
  columns,
  rows,
  renderBounds,
  fps,
  themeMappings,
  framesByTheme: matrixFramesByTheme,
});

console.log(
  `Generated ${matrixFramesByTheme.dark.length} matrix frames per theme (${columns}x${rows}, render ${renderBounds.width}x${renderBounds.height} at ${renderBounds.x},${renderBounds.y}, ${levels} levels, gamma ${gamma}, contrast ${contrast}${autoLevels ? ", auto levels" : ""}) -> ${path.relative(process.cwd(), path.resolve(output))}, ${path.relative(process.cwd(), path.resolve(dataOutput))}`,
);

function parseArgs(args) {
  const parsed = {};

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];

    if (arg === "--") {
      continue;
    }

    if (!arg.startsWith("--")) {
      parsed.input ??= arg;
      continue;
    }

    const [rawKey, inlineValue] = arg.slice(2).split("=", 2);
    const key = rawKey.replaceAll("-", "_");

    if (key === "invert" || key === "auto_levels" || key === "auto_contrast") {
      parsed[key === "auto_contrast" ? "auto_levels" : key] = true;
      continue;
    }

    const value = inlineValue ?? args[index + 1];
    if (inlineValue === undefined) {
      index += 1;
    }

    switch (key) {
      case "input":
        parsed.input = value;
        break;
      case "output":
        parsed.output = value;
        break;
      case "data_output":
      case "data":
        parsed.data_output = value;
        break;
      case "columns":
      case "cols":
        parsed.columns = value;
        break;
      case "rows":
        parsed.rows = value;
        break;
      case "fps":
        parsed.fps = value;
        break;
      case "frames":
      case "max_frames":
        parsed.frames = value;
        break;
      case "levels":
        parsed.levels = value;
        break;
      case "contrast":
        parsed.contrast = value;
        break;
      case "gamma":
        parsed.gamma = value;
        break;
      case "brightness":
        parsed.brightness = value;
        break;
      case "black_point":
      case "black":
        parsed.black_point = value;
        break;
      case "white_point":
      case "white":
        parsed.white_point = value;
        break;
      case "min_level":
      case "density":
        parsed.min_level = value;
        break;
      case "threshold":
        parsed.threshold = value;
        break;
      default:
        throw new Error(`Unknown option: --${rawKey}`);
    }
  }

  return parsed;
}

function positiveInteger(value, fallback, name) {
  if (value === undefined) {
    return fallback;
  }

  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`--${name} must be a positive integer`);
  }

  return parsed;
}

function positiveNumber(value, fallback, name) {
  if (value === undefined) {
    return fallback;
  }

  const parsed = Number.parseFloat(value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`--${name} must be a positive number`);
  }

  return parsed;
}

function numberOption(value, fallback, name) {
  if (value === undefined) {
    return fallback;
  }

  const parsed = Number.parseFloat(value);
  if (!Number.isFinite(parsed)) {
    throw new Error(`--${name} must be a number`);
  }

  return parsed;
}

async function readMediaSize(input) {
  const ffprobe = spawn(
    "ffprobe",
    [
      "-v",
      "error",
      "-select_streams",
      "v:0",
      "-show_entries",
      "stream=width,height",
      "-of",
      "json",
      input,
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );

  const stdoutChunks = [];
  const stderrChunks = [];
  ffprobe.stdout.on("data", (chunk) => stdoutChunks.push(chunk));
  ffprobe.stderr.on("data", (chunk) => stderrChunks.push(chunk));

  const exitCode = await waitForProcess(ffprobe, "ffprobe");
  if (exitCode !== 0) {
    const message = Buffer.concat(stderrChunks).toString("utf8").trim();
    throw new Error(
      `ffprobe exited with code ${exitCode}${message ? `: ${message}` : ""}`,
    );
  }

  const parsed = JSON.parse(Buffer.concat(stdoutChunks).toString("utf8"));
  const stream = parsed.streams?.[0];

  if (!stream?.width || !stream?.height) {
    throw new Error("Could not read source video dimensions with ffprobe.");
  }

  return {
    width: stream.width,
    height: stream.height,
  };
}

function containInGrid({ sourceWidth, sourceHeight, columns, rows }) {
  const sourceAspect = sourceWidth / sourceHeight;
  const gridAspect = columns / rows;

  if (sourceAspect > gridAspect) {
    return {
      width: columns,
      height: Math.max(1, Math.min(rows, Math.round(columns / sourceAspect))),
    };
  }

  return {
    width: Math.max(1, Math.min(columns, Math.round(rows * sourceAspect))),
    height: rows,
  };
}

async function readRawGrayFrames({
  input,
  columns,
  rows,
  outputSize,
  fps,
  maxFrames,
}) {
  const ffmpeg = spawn(
    "ffmpeg",
    [
      "-hide_banner",
      "-loglevel",
      "error",
      "-i",
      input,
      "-vf",
      `fps=${fps},scale=${outputSize.width}:${outputSize.height},pad=${columns}:${rows}:(ow-iw)/2:(oh-ih)/2,format=gray`,
      "-frames:v",
      String(maxFrames),
      "-f",
      "rawvideo",
      "-pix_fmt",
      "gray",
      "pipe:1",
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );

  const stderrChunks = [];
  ffmpeg.stderr.on("data", (chunk) => stderrChunks.push(chunk));

  const frames = [];
  let pending = Buffer.alloc(0);

  ffmpeg.stdout.on("data", (chunk) => {
    pending = Buffer.concat([pending, chunk]);

    while (pending.length >= frameSize) {
      frames.push(pending.subarray(0, frameSize));
      pending = pending.subarray(frameSize);
    }
  });

  const exitCode = await waitForProcess(ffmpeg, "ffmpeg");
  if (exitCode !== 0) {
    const message = Buffer.concat(stderrChunks).toString("utf8").trim();
    throw new Error(
      `ffmpeg exited with code ${exitCode}${message ? `: ${message}` : ""}`,
    );
  }

  return frames;
}

function waitForProcess(childProcess, command) {
  return new Promise((resolve, reject) => {
    childProcess.on("error", reject);
    childProcess.on("close", resolve);
  }).catch((error) => {
    if (error?.code === "ENOENT") {
      throw new Error(
        `${command} was not found on PATH. Install ffmpeg, then run this generator again.`,
      );
    }

    throw error;
  });
}

function frameToMatrixString(frame, columns, levels, mapping, renderBounds) {
  const frameLevels = mapping.autoLevels
    ? levelsForFrame(frame)
    : { min: mapping.blackPoint, max: mapping.whitePoint };
  let encoded = "";

  for (let index = 0; index < frame.length; index += 1) {
    if (!isInsideRenderBounds(index, columns, renderBounds)) {
      encoded += ENCODING[0];
      continue;
    }

    const brightness = frame[index];
    const normalized = normalizeBrightness(brightness, frameLevels, mapping);
    const level =
      mapping.threshold === undefined
        ? Math.round(normalized * (levels - 1))
        : normalized >= mapping.threshold
          ? levels - 1
          : 0;

    encoded += ENCODING[level];
  }

  return encoded;
}

function isInsideRenderBounds(index, columns, renderBounds) {
  const row = Math.floor(index / columns);
  const column = index % columns;

  return (
    column >= renderBounds.x &&
    column < renderBounds.x + renderBounds.width &&
    row >= renderBounds.y &&
    row < renderBounds.y + renderBounds.height
  );
}

function levelsForFrame(frame) {
  const histogram = new Array(256).fill(0);
  for (const value of frame) {
    histogram[value] += 1;
  }

  const total = frame.length;
  const lowCutoff = total * 0.02;
  const highCutoff = total * 0.98;
  let cumulative = 0;
  let min = 0;
  let max = 255;

  for (let index = 0; index < histogram.length; index += 1) {
    cumulative += histogram[index];
    if (cumulative >= lowCutoff) {
      min = index;
      break;
    }
  }

  cumulative = 0;
  for (let index = 0; index < histogram.length; index += 1) {
    cumulative += histogram[index];
    if (cumulative >= highCutoff) {
      max = index;
      break;
    }
  }

  if (max <= min) {
    return { min: 0, max: 255 };
  }

  return { min, max };
}

function normalizeBrightness(brightness, levels, mapping) {
  const span = Math.max(1, levels.max - levels.min);
  let value = (brightness - levels.min) / span;

  value = clamp((value - 0.5) * mapping.contrast + 0.5);
  value = clamp(value + mapping.brightness);
  value = clamp(Math.pow(value, mapping.gamma));

  if (mapping.invert) {
    value = 1 - value;
  }

  if (mapping.minLevel > 0) {
    value = mapping.minLevel + value * (1 - mapping.minLevel);
  }

  return clamp(value);
}

function clamp(value) {
  return Math.max(0, Math.min(1, value));
}

async function writeFrameModule({
  output,
  dataOutput,
  source,
  columns,
  rows,
  renderBounds,
  fps,
  themeMappings,
  framesByTheme,
}) {
  const resolvedOutput = path.resolve(output);
  const resolvedDataOutput = path.resolve(dataOutput);
  await mkdir(path.dirname(resolvedOutput), { recursive: true });
  await mkdir(path.dirname(resolvedDataOutput), { recursive: true });

  const sourceName = path.basename(source);
  const dataUrl = publicAssetUrl(resolvedDataOutput);
  const dataBody = `${JSON.stringify(
    {
      source: sourceName,
      framesByTheme,
    },
    null,
    2,
  )}
`;
  const body = `// Generated by scripts/generate_matrix_frames.mjs from ${JSON.stringify(sourceName)}.
// Do not edit by hand; regenerate this file from source media instead.

export const matrixFramesUrl = ${JSON.stringify(dataUrl)};
export const matrixFrameRate = ${fps};
export const matrixColumns = ${columns};
export const matrixRows = ${rows};
export const matrixRenderBounds = ${JSON.stringify(renderBounds, null, 2)} as const;
export const matrixLevels = ${themeMappings.dark.levels};
export const matrixEncoding = ${JSON.stringify(ENCODING.slice(0, themeMappings.dark.levels))};
export const matrixThemeMappings = ${JSON.stringify(themeMappings, null, 2)} as const;
`;

  await writeFile(resolvedDataOutput, dataBody, "utf8");
  await writeFile(resolvedOutput, body, "utf8");
}

function publicAssetUrl(filePath) {
  const publicRoot = path.resolve(REPO_ROOT, "apps/desktop/public");
  const relativePath = path.relative(publicRoot, filePath);

  if (relativePath.startsWith("..") || path.isAbsolute(relativePath)) {
    throw new Error(
      `--data-output must live under ${publicRoot} so Vite can serve it as a static asset.`,
    );
  }

  return `/${relativePath.split(path.sep).join("/")}`;
}

function printUsage() {
  console.log(`Usage:
  node scripts/generate_matrix_frames.mjs --input path/to/source.gif

Options:
  --input <path>       Source video or GIF. You can also pass this as the first positional arg.
  --output <path>      Generated TypeScript metadata file. Default: ${DEFAULT_OUTPUT}
  --data-output <path> Generated static frame data JSON. Default: ${DEFAULT_DATA_OUTPUT}
  --columns <number>   Matrix frame width. Default: 120
  --rows <number>      Matrix frame height. Default: 64
  --levels <2-36>      Brightness levels per cell. Default: 12
  --auto-levels        Stretch each frame's useful brightness range.
  --contrast <number>  Contrast multiplier. Default: 1
  --gamma <number>     Gamma curve. Default: 1
  --brightness <n>     Brightness offset from -1 to 1. Default: 0
  --min-level <0-1>    Minimum brightness level for non-binary output. Default: 0
  --threshold <0-1>    Emit binary 0/full cells using this threshold.
  --fps <number>       Output animation FPS. Default: 12
  --frames <number>    Max frames to generate. Default: 180
  --invert             Invert brightness.

Example:
  node scripts/generate_matrix_frames.mjs intro.mp4 --columns 140 --rows 72 --levels 14 --fps 10 --frames 96 --auto-levels --gamma 0.75 --contrast 1.2
`);
}
