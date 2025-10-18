import { Card, cn, Hotkey, Input } from "@skopio/ui";
import { PartyPopper } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { commands, NotificationPayload } from "@/types/tauri.gen";
import { isDev } from "@/utils/environment";
import { resolveResource } from "@tauri-apps/api/path";
import { convertFileSrc } from "@tauri-apps/api/core";
import confetti from "canvas-confetti";

export const NotificationPanel = () => {
  const invisibleInputRef = useRef<HTMLInputElement | null>(null);
  const [isExiting, setIsExiting] = useState(false);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const iconRef = useRef<SVGSVGElement | null>(null);

  const [payload] = useState<NotificationPayload | null>(() => {
    if (typeof window === "undefined") return null;
    try {
      const urlParams = new URLSearchParams(window.location.search);
      const raw = urlParams.get("payload");
      if (!raw) return null;
      const decoded = decodeURIComponent(raw);
      return JSON.parse(decoded) as NotificationPayload;
    } catch (err) {
      console.error("Failed to decode payload:", err);
      return null;
    }
  });

  useEffect(() => {
    if (!payload || !iconRef.current) return;

    const rect = iconRef.current.getBoundingClientRect();
    const x = (rect.left + rect.width) / window.innerWidth;
    const y = (rect.top + rect.height) / window.innerHeight;

    confetti({
      particleCount: 600,
      spread: 85,
      origin: { x, y },
      zIndex: 9999,
    });
  }, [payload]);

  const dismiss = () => {
    setIsExiting(true);
    setTimeout(() => {
      commands.dismissNotificationWindow().catch(console.error);
    }, 500);
  };

  const getSoundPath = async (soundFile: string) => {
    if (isDev()) {
      return `/src-tauri/sounds/${soundFile}`;
    }
    const resourcePath = await resolveResource(`sounds/${soundFile}`);
    return convertFileSrc(resourcePath);
  };

  const playSound = useCallback(async () => {
    if (!payload?.soundFile) return;
    const soundPath = await getSoundPath(payload.soundFile);
    const audio = new Audio(soundPath);
    audioRef.current = audio;

    const handleCanPlay = () => {
      audio.play().catch((err) => console.error("Error playing sound: ", err));
    };

    audio.addEventListener("canplaythrough", handleCanPlay, { once: true });
  }, [payload]);

  useEffect(() => {
    if (!payload) return;

    invisibleInputRef.current?.focus();

    const timeout = setTimeout(dismiss, payload?.durationMs);
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") dismiss();
    };

    window.addEventListener("keydown", handleKey);
    playSound();

    return () => {
      clearTimeout(timeout);
      window.removeEventListener("keydown", handleKey);
      if (audioRef.current) {
        audioRef.current.pause();
        audioRef.current = null;
      }
    };
  }, [payload, playSound]);

  if (!payload) return null;

  return (
    <div className="min-h-screen font-sans">
      <Input
        ref={invisibleInputRef}
        className="absolute opacity-0 pointer-events-none w-0 h-0 border-0 outline-none"
        style={{ position: "absolute", left: "-1000px" }}
        tabIndex={-1}
        autoFocus
      />

      <Card
        className={cn(
          "group relative flex items-center gap-3 p-4 bg-white/70 backdrop-blur-sm shadow-lg border-border",
          "animate-in slide-in-from-top-4 fade-in duration-300",
          isExiting && "animate-out slide-out-top-4 fade-out duration-500",
        )}
      >
        <PartyPopper
          ref={iconRef}
          className="h-6 w-6 shrink-0 text-green-500"
        />

        <div className="flex-1">
          <h3 className="text-card-foreground font-semibold text-base">
            {payload.title}
          </h3>
          <p className="text-card-foreground text-sm">{payload.message}</p>
        </div>

        <div
          className="absolute -top-2 -left-2 transition-opacity cursor-pointer opacity-0 group-hover:opacity-100"
          onClick={dismiss}
        >
          <Hotkey size="sm" color="red">
            ESC
          </Hotkey>
        </div>

        <div className="absolute bottom-0 left-1 right-1 h-0.5 bg-primary/20 rounded-full">
          <div
            className="h-full rounded-full origin-left bg-green-500"
            style={{
              animation: `progress-shrink ${payload.durationMs}ms linear forwards`,
            }}
          />
        </div>
      </Card>

      <style>
        {`
            @keyframes progress-shrink {
                from {
                   transform: scaleX(1);
                }
                to {
                  transform: scaleX(0);
                }
            }
        `}
      </style>
    </div>
  );
};
