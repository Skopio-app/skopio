import { Card, cn, Hotkey, Input } from "@skopio/ui";
import { PartyPopper } from "lucide-react";
import { useRef, useState } from "react";

export const NotificationPanel = () => {
  const invisibleInputRef = useRef<HTMLInputElement | null>(null);
  const [isExiting, setIsExiting] = useState(false);

  const handleExit = () => {
    setIsExiting(false);
  };

  return (
    <div className="min-h-screen font-sans p-2">
      <Input
        ref={invisibleInputRef}
        className="absolute opacity-0 pointer-events-none w-0 h-0 border-0 outline-none"
        style={{ position: "absolute", left: "-1000px" }}
        tabIndex={-1}
        autoFocus
      />

      <Card
        className={cn(
          "group relative flex items-center gap-3 p-4 bg-card/95 backdrop-blur-sm shadow-lg border-border",
          "animate-in slide-in-from-top-4 fade-in duration-300",
          isExiting && "animate-out slide-out-top-4 fade-out duration-500",
        )}
      >
        <PartyPopper className="h-6 w-6 shrink-0 text-green-500" />

        <div className="flex-1">
          <h3 className="text-card-foreground font-semibold text-base">
            Goal achieved!
          </h3>
          <p className="text-card-foreground text-sm">
            You have achieved your goal
          </p>
        </div>

        <div
          className="absolute -top-2 -left-2 transition-opacity cursor-pointer opacity-0 group-hover:opacity-100"
          onClick={handleExit}
        >
          <Hotkey size="sm" color="default">
            ESC
          </Hotkey>
        </div>

        <div className="absolute bottom-0 left-1 right-1 h-0.5 bg-primary/20 rounded-full">
          <div
            className="h-full rounded-full origin-left bg-green-500"
            style={{
              animation: "progress-shrink 5000ms linear forwards",
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
