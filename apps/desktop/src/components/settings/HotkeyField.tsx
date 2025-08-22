import { useEffect, useRef, useState } from "react";
import Kbd from "./kbd";
import { cn } from "@skopio/ui";

const MODS: Record<string, string> = {
  Meta: "⌘",
  Control: "Ctrl",
  Alt: "⌥",
  Shift: "⇧",
};

const formatCombo = (parts: string[]): string => {
  return parts.join("+");
};

const parseKeyForDisplay = (key: string) => {
  if (key === " ") return "Space";
  if (key === "ArrowUp") return "↑";
  if (key === "ArrowDown") return "↓";
  if (key === "ArrowLeft") return "←";
  if (key === "ArrowRight") return "→";
  if (key.length === 1) return key.toUpperCase();
  return key;
};

const HotkeyField: React.FC<{
  value?: string;
  onChange?: (val: string) => void;
  placeholder?: string;
}> = ({ value, onChange, placeholder }) => {
  const [recording, setRecording] = useState(false);
  const containerRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!recording) return;
    const handlePointerDown = (e: MouseEvent | PointerEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setRecording(false);
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    return () =>
      document.removeEventListener("pointerdown", handlePointerDown, true);
  }, [recording]);

  useEffect(() => {
    if (!recording) return;

    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();

      const parts: string[] = [];
      if (e.metaKey) parts.push(MODS.Meta);
      if (e.ctrlKey) parts.push(MODS.Control);
      if (e.altKey) parts.push(MODS.Alt);
      if (e.shiftKey) parts.push(MODS.Shift);

      const isPureMod =
        e.key === "Meta" ||
        e.key === "Control" ||
        e.key === "Alt" ||
        e.key === "Shift";

      if (!isPureMod) {
        parts.push(parseKeyForDisplay(e.key));
        const combo = formatCombo(parts);
        onChange?.(combo);
        setRecording(false);
      }
    };

    window.addEventListener("keydown", handler, { capture: true });
    return () =>
      window.removeEventListener("keydown", handler, { capture: true } as any);
  }, [recording, onChange]);

  return (
    <div
      ref={containerRef}
      className={cn(
        "flex h-10 w-full items-center justify-between rounded-md border px-3",
        recording ? "ring-2 ring-primary" : "bg-background",
      )}
      role="button"
      aria-label="Set global shortcut"
      onClick={() => setRecording(true)}
    >
      <div className="flex flex-wrap items-center gap-1 py-1">
        {value ? (
          value
            .split("+")
            .map((token, i) => <Kbd key={`${token}-${i}`}>{token}</Kbd>)
        ) : (
          <span className="text-sm text-muted-foreground">
            {placeholder ?? "Click to record"}
          </span>
        )}
      </div>
    </div>
  );
};

export default HotkeyField;
