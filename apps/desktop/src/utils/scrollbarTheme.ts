import { useEffect } from "react";

const SCROLLBAR_THUMB = "--scrollbar-subtle-thumb";
const SCROLLBAR_THUMB_HOVER = "--scrollbar-subtle-thumb-hover";

const clamp = (value: number, min = 0, max = 1) =>
  Math.min(max, Math.max(min, value));

const parseAlpha = (alpha: string | undefined) => {
  if (!alpha) return 1;
  const trimmed = alpha.trim();

  if (trimmed.endsWith("%")) {
    return clamp(Number(trimmed.slice(0, -1)) / 100);
  }

  return clamp(Number(trimmed));
};

const formatRgb = (red: number, green: number, blue: number, alpha = 1) => {
  const r = Math.round(clamp(red) * 255);
  const g = Math.round(clamp(green) * 255);
  const b = Math.round(clamp(blue) * 255);

  if (alpha >= 1) return `rgb(${r}, ${g}, ${b})`;
  return `rgba(${r}, ${g}, ${b}, ${alpha.toFixed(3)})`;
};

const linearToSrgb = (channel: number) =>
  channel <= 0.0031308 ? 12.92 * channel : 1.055 * channel ** (1 / 2.4) - 0.055;

const labToRgb = (lightness: number, a: number, b: number, alpha = 1) => {
  const epsilon = 216 / 24389;
  const kappa = 24389 / 27;

  const fy = (lightness + 16) / 116;
  const fx = fy + a / 500;
  const fz = fy - b / 200;

  const xr = fx ** 3 > epsilon ? fx ** 3 : (116 * fx - 16) / kappa;
  const yr =
    lightness > kappa * epsilon
      ? ((lightness + 16) / 116) ** 3
      : lightness / kappa;
  const zr = fz ** 3 > epsilon ? fz ** 3 : (116 * fz - 16) / kappa;

  const xD50 = xr * 0.96422;
  const yD50 = yr;
  const zD50 = zr * 0.82521;

  const x = 0.9555766 * xD50 - 0.0230393 * yD50 + 0.0631636 * zD50;
  const y = -0.0282895 * xD50 + 1.0099416 * yD50 + 0.0210077 * zD50;
  const z = 0.0122982 * xD50 - 0.020483 * yD50 + 1.3299098 * zD50;

  const red = linearToSrgb(3.2404542 * x - 1.5371385 * y - 0.4985314 * z);
  const green = linearToSrgb(-0.969266 * x + 1.8760108 * y + 0.041556 * z);
  const blue = linearToSrgb(0.0556434 * x - 0.2040259 * y + 1.0572252 * z);

  return formatRgb(red, green, blue, alpha);
};

const cssLabToRgb = (color: string) => {
  const match = color
    .trim()
    .match(
      /^lab\(\s*([+-]?\d*\.?\d+)%?\s+([+-]?\d*\.?\d+)\s+([+-]?\d*\.?\d+)(?:\s*\/\s*([^)]+))?\s*\)$/i,
    );

  if (!match) return null;

  return labToRgb(
    Number(match[1]),
    Number(match[2]),
    Number(match[3]),
    parseAlpha(match[4]),
  );
};

const isScrollbarSafeColor = (color: string) =>
  /^(rgb|rgba|hsl|hsla)\(/i.test(color.trim()) || color.trim().startsWith("#");

const readResolvedColor = (variableName: string) => {
  const probe = document.createElement("span");
  probe.style.color = `var(${variableName})`;
  probe.style.pointerEvents = "none";
  probe.style.position = "fixed";
  probe.style.visibility = "hidden";
  document.body.appendChild(probe);

  const color = window.getComputedStyle(probe).color.trim();
  probe.remove();

  if (isScrollbarSafeColor(color)) return color;

  return (
    cssLabToRgb(color) ??
    cssLabToRgb(
      window
        .getComputedStyle(document.documentElement)
        .getPropertyValue(variableName)
        .trim(),
    )
  );
};

export const syncScrollbarThemeColors = () => {
  const root = document.documentElement;
  const thumb = readResolvedColor("--muted-foreground");
  const hover = readResolvedColor("--foreground");

  if (thumb) root.style.setProperty(SCROLLBAR_THUMB, thumb);
  if (hover) root.style.setProperty(SCROLLBAR_THUMB_HOVER, hover);
};

export const useScrollbarThemeColors = () => {
  useEffect(() => {
    const sync = () => window.requestAnimationFrame(syncScrollbarThemeColors);
    const observer = new MutationObserver(sync);

    sync();
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    return () => {
      observer.disconnect();
    };
  }, []);
};
