import { cn } from "@skopio/ui";
import {
  CSSProperties,
  PointerEvent as ReactPointerEvent,
  ReactNode,
  useCallback,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";

type SubtleScrollAreaProps = {
  children: ReactNode;
  className?: string;
  viewportClassName?: string;
};

type ThumbState = {
  visible: boolean;
  top: number;
  height: number;
};

const MIN_THUMB_HEIGHT = 32;

export const SubtleScrollArea = ({
  children,
  className,
  viewportClassName,
}: SubtleScrollAreaProps) => {
  const viewportRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<{
    pointerId: number;
    startY: number;
    startScrollTop: number;
  } | null>(null);
  const [thumb, setThumb] = useState<ThumbState>({
    visible: false,
    top: 0,
    height: MIN_THUMB_HEIGHT,
  });

  const updateThumb = useCallback(() => {
    const viewport = viewportRef.current;
    if (!viewport) return;

    const { scrollHeight, clientHeight, scrollTop } = viewport;
    const maxScrollTop = scrollHeight - clientHeight;

    if (maxScrollTop <= 0) {
      setThumb((current) =>
        current.visible
          ? { visible: false, top: 0, height: MIN_THUMB_HEIGHT }
          : current,
      );
      return;
    }

    const height = Math.max(
      MIN_THUMB_HEIGHT,
      (clientHeight / scrollHeight) * clientHeight,
    );
    const top = (scrollTop / maxScrollTop) * (clientHeight - height);

    setThumb({ visible: true, top, height });
  }, []);

  useLayoutEffect(() => {
    updateThumb();
  }, [updateThumb, children]);

  useEffect(() => {
    const viewport = viewportRef.current;
    if (!viewport) return;

    const resizeObserver = new ResizeObserver(updateThumb);
    resizeObserver.observe(viewport);

    if (viewport.firstElementChild) {
      resizeObserver.observe(viewport.firstElementChild);
    }

    viewport.addEventListener("scroll", updateThumb, { passive: true });
    window.addEventListener("resize", updateThumb);

    return () => {
      resizeObserver.disconnect();
      viewport.removeEventListener("scroll", updateThumb);
      window.removeEventListener("resize", updateThumb);
    };
  }, [updateThumb]);

  const onThumbPointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    const viewport = viewportRef.current;
    if (!viewport) return;

    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = {
      pointerId: event.pointerId,
      startY: event.clientY,
      startScrollTop: viewport.scrollTop,
    };
  };

  const onThumbPointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    const viewport = viewportRef.current;
    const drag = dragRef.current;
    if (!viewport || !drag || drag.pointerId !== event.pointerId) return;

    const maxScrollTop = viewport.scrollHeight - viewport.clientHeight;
    const maxThumbTop = viewport.clientHeight - thumb.height;
    const scrollPerPixel = maxThumbTop > 0 ? maxScrollTop / maxThumbTop : 0;

    viewport.scrollTop =
      drag.startScrollTop + (event.clientY - drag.startY) * scrollPerPixel;
  };

  const stopDragging = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (dragRef.current?.pointerId === event.pointerId) {
      dragRef.current = null;
    }
  };

  return (
    <div className={cn("scrollbar-subtle relative min-h-0 min-w-0", className)}>
      <div
        ref={viewportRef}
        className={cn(
          "h-full w-full overflow-auto scroll-hidden",
          viewportClassName,
        )}
      >
        {children}
      </div>

      {thumb.visible && (
        <div className="pointer-events-none absolute inset-y-0 right-0 w-3">
          <div
            className="pointer-events-auto absolute right-1 w-1.5 cursor-default rounded-full bg-(--muted-foreground)"
            style={
              {
                height: thumb.height,
                transform: `translateY(${thumb.top}px)`,
              } satisfies CSSProperties
            }
            onPointerDown={onThumbPointerDown}
            onPointerMove={onThumbPointerMove}
            onPointerCancel={stopDragging}
            onPointerUp={stopDragging}
          />
        </div>
      )}
    </div>
  );
};
