import { useEffect, useRef, useState } from "react";
import Kbd from "@/components/settings/kbd";
import { cn, Popover, PopoverContent, PopoverTrigger } from "@skopio/ui";
import { baseKeyFromEvent } from "@/utils/hotkey";
import { Check } from "lucide-react";

enum Mods {
  Meta = "⌘",
  Control = "Ctrl",
  Alt = "⌥",
  Shift = "⇧",
}

interface HotkeyFieldProps {
  value?: string;
  onChange?: (val: string) => void;
  placeholder?: string;
}

const HotkeyField: React.FC<HotkeyFieldProps> = ({
  value,
  onChange,
  placeholder,
}) => {
  const [recording, setRecording] = useState(false);
  const [open, setOpen] = useState(false);
  const [showCheck, setShowCheck] = useState(false);
  const [liveParts, setLiveParts] = useState<string[]>([]);
  const containerRef = useRef<HTMLDivElement | null>(null);
  const finalizeTimer = useRef<number | null>(null);

  useEffect(() => {
    if (!open) return;
    const handlePointerDown = (e: MouseEvent | PointerEvent) => {
      if (
        containerRef.current &&
        !containerRef.current.contains(e.target as Node)
      ) {
        setRecording(false);
        setOpen(false);
        setLiveParts([]);
        setShowCheck(false);
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    return () =>
      document.removeEventListener("pointerdown", handlePointerDown, true);
  }, [open]);

  useEffect(() => {
    if (!recording) return;

    const handler = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.repeat) return;

      const parts: string[] = [];
      if (e.metaKey) parts.push(Mods.Meta);
      if (e.ctrlKey) parts.push(Mods.Control);
      if (e.altKey || e.getModifierState?.("AltGraph")) parts.push(Mods.Alt);
      if (e.shiftKey) parts.push(Mods.Shift);

      const isPureMod =
        e.key === "Meta" ||
        e.key === "Control" ||
        e.key === "Alt" ||
        e.key === "Shift";

      if (isPureMod) {
        setLiveParts(parts);
        return;
      }

      const base = baseKeyFromEvent(e);
      if (!base) return;

      const comboParts = [...parts, base];
      setLiveParts(comboParts);

      const combo = comboParts.join("+");
      onChange?.(combo);

      setShowCheck(true);
      if (finalizeTimer.current) window.clearTimeout(finalizeTimer.current);
      finalizeTimer.current = window.setTimeout(() => {
        setShowCheck(false);
        setOpen(false);
        setRecording(false);
        setLiveParts([]);
      }, 700) as number;
    };

    window.addEventListener("keydown", handler, { capture: true });
    return () => {
      window.removeEventListener("keydown", handler, { capture: true } as any);
      if (finalizeTimer.current) window.clearTimeout(finalizeTimer.current);
    };
  }, [recording, onChange]);

  return (
    <Popover
      open={open}
      onOpenChange={(v) => {
        setOpen(v);
        if (!v) {
          setRecording(false);
          setLiveParts([]);
          setShowCheck(false);
        }
      }}
    >
      <PopoverTrigger asChild>
        <div
          ref={containerRef}
          className={cn(
            "flex h-10 w-full items-center justify-between rounded-md border px-3 cursor-pointer",
            recording ? "ring-2 ring-primary" : "bg-background",
          )}
          role="button"
          aria-label="Set global shortcut"
          onClick={() => {
            setRecording(true);
            setOpen(true);
            setLiveParts([]);
            setShowCheck(false);
          }}
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
      </PopoverTrigger>

      <PopoverContent align="start" side="bottom" className="w-64">
        <div className="relative">
          {showCheck && (
            <div className="absolute right-0 top-0 p-1">
              <Check className="w-4 h-4 text-green-600" />
            </div>
          )}
          <div className="flex min-h-10 items-center gap-2 pr-6">
            {liveParts.length === 0 ? (
              <span className="text-sm text-muted-foreground">
                Recording...
              </span>
            ) : (
              <div className="flex flex-wrap items-center gap-1">
                {liveParts.map((t, i) => (
                  <Kbd key={`${t}-${i}`}>{t}</Kbd>
                ))}
              </div>
            )}
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            Hold modifiers (⌘/Ctrl/⌥/⇧), then press a key to set the shortcut.
          </p>
        </div>
      </PopoverContent>
    </Popover>
  );
};

export default HotkeyField;
