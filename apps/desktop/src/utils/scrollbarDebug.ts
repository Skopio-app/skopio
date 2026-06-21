import { useEffect } from "react";

declare global {
  interface Window {
    __skopioDebugScrollbars?: () => void;
  }
}

const DEBUG_FLAG = "skopio.debug.scrollbars";

const isScrollableY = (element: Element) => {
  if (!(element instanceof HTMLElement)) return false;

  const style = window.getComputedStyle(element);
  return (
    (style.overflowY === "auto" || style.overflowY === "scroll") &&
    element.scrollHeight > element.clientHeight
  );
};

export const debugScrollbars = () => {
  const rootStyle = window.getComputedStyle(document.documentElement);
  const subtleElements = Array.from(
    document.querySelectorAll<HTMLElement>(".scrollbar-subtle"),
  );
  const scrollableElements = Array.from(
    document.querySelectorAll<HTMLElement>("*"),
  ).filter(isScrollableY);

  console.groupCollapsed("[Skopio] scrollbar debug");
  console.info("document theme classes", document.documentElement.className);
  console.info(
    "root --muted-foreground",
    rootStyle.getPropertyValue("--muted-foreground").trim(),
  );
  console.info(
    "root --foreground",
    rootStyle.getPropertyValue("--foreground").trim(),
  );
  console.info(
    "root --scrollbar-subtle-thumb",
    rootStyle.getPropertyValue("--scrollbar-subtle-thumb").trim(),
  );
  console.info(
    "root --scrollbar-subtle-thumb-hover",
    rootStyle.getPropertyValue("--scrollbar-subtle-thumb-hover").trim(),
  );
  console.info("scrollbar-subtle count", subtleElements.length);
  console.info("scrollable element count", scrollableElements.length);

  subtleElements.forEach((element, index) => {
    const style = window.getComputedStyle(element);
    const scrollbar = window.getComputedStyle(element, "::-webkit-scrollbar");
    const thumb = window.getComputedStyle(element, "::-webkit-scrollbar-thumb");
    const track = window.getComputedStyle(element, "::-webkit-scrollbar-track");

    console.info(`.scrollbar-subtle[${index}]`, {
      element,
      className: element.className,
      overflowY: style.overflowY,
      scrollHeight: element.scrollHeight,
      clientHeight: element.clientHeight,
      scrollTop: element.scrollTop,
      scrollbarColor: style.scrollbarColor,
      scrollbarWidth: style.scrollbarWidth,
      subtleThumb: style.getPropertyValue("--scrollbar-subtle-thumb").trim(),
      webkitScrollbarWidth: scrollbar.width,
      webkitScrollbarHeight: scrollbar.height,
      webkitThumbBackgroundColor: thumb.backgroundColor,
      webkitThumbBackgroundClip: thumb.backgroundClip,
      webkitTrackBackgroundColor: track.backgroundColor,
    });
  });

  scrollableElements.forEach((element, index) => {
    console.info(`scrollable[${index}]`, {
      element,
      className: element.className,
      hasSubtleClass: element.classList.contains("scrollbar-subtle"),
      insideSubtle: Boolean(element.closest(".scrollbar-subtle")),
      overflowY: window.getComputedStyle(element).overflowY,
      scrollHeight: element.scrollHeight,
      clientHeight: element.clientHeight,
    });
  });

  console.groupEnd();
};

export const useScrollbarDebugProbe = () => {
  useEffect(() => {
    window.__skopioDebugScrollbars = debugScrollbars;

    if (localStorage.getItem(DEBUG_FLAG) === "1") {
      window.requestAnimationFrame(debugScrollbars);
    }

    return () => {
      delete window.__skopioDebugScrollbars;
    };
  }, []);
};
