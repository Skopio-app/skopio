import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  ACTIONS,
  Joyride,
  EVENTS,
  STATUS,
  type EventData,
} from "react-joyride";
import { useLocation, useNavigate } from "react-router";
import { commands } from "@/types/tauri.gen";
import { TourContext } from "./TourContext";
import { TOUR_STEPS, type TourId } from "./TourSteps";

const TOUR_COMPLETED_KEY = "skopio.tour.main.completed";
const PENDING_SETTINGS_TOUR_KEY = "skopio.tour.settings.pending";
const RESTART_MAIN_TOUR_KEY = "skopio.tour.main.restartAt";
const STARTUP_PERMISSION_DIALOG_SELECTOR = "[data-startup-permissions-dialog]";
const TOOLTIP_VIEWPORT_PADDING = {
  top: 56,
  right: 24,
  bottom: 24,
  left: 24,
};

const waitForTarget = (selector: string) =>
  new Promise<void>((resolve) => {
    if (document.querySelector(selector)) {
      resolve();
      return;
    }

    const observer = new MutationObserver(() => {
      if (document.querySelector(selector)) {
        observer.disconnect();
        resolve();
      }
    });

    observer.observe(document.body, { childList: true, subtree: true });

    window.setTimeout(() => {
      observer.disconnect();
      resolve();
    }, 2500);
  });

export function TourProvider({ children }: { children: ReactNode }) {
  const navigate = useNavigate();
  const location = useLocation();
  const isSettingsWindow = location.pathname.startsWith("/settings");

  const [tourId, setTourId] = useState<TourId>("main");
  const [run, setRun] = useState(false);
  const [runKey, setRunKey] = useState(0);
  const [windowSignal, setWindowSignal] = useState(0);
  const floatingOptions = useMemo(() => {
    const viewportBoundary =
      typeof document === "undefined" ? undefined : document.documentElement;

    return {
      strategy: "fixed" as const,
      autoUpdate: {
        ancestorScroll: true,
        elementResize: true,
        animationFrame: true,
      },
      flipOptions: {
        boundary: viewportBoundary,
        rootBoundary: "viewport" as const,
        padding: TOOLTIP_VIEWPORT_PADDING,
        crossAxis: true,
      },
      shiftOptions: {
        boundary: viewportBoundary,
        rootBoundary: "viewport" as const,
        padding: TOOLTIP_VIEWPORT_PADDING,
        crossAxis: true,
      },
    };
  }, []);

  const steps = useMemo(
    () =>
      TOUR_STEPS[tourId].map((step) => {
        const target = `[data-tour="${step.target}"]`;

        return {
          target,
          title: step.title,
          content: step.body,
          placement: step.placement ?? "auto",
          spotlightTarget: step.spotlightTarget
            ? `[data-tour="${step.spotlightTarget}"]`
            : undefined,
          scrollTarget: step.scrollTarget
            ? `[data-tour="${step.scrollTarget}"]`
            : undefined,
          before: async () => {
            if (step.route && window.location.pathname !== step.route) {
              navigate(step.route);
            }

            await waitForTarget(target);
          },
        };
      }),
    [navigate, tourId],
  );

  const runTour = useCallback((nextTour: TourId) => {
    setTourId(nextTour);
    setRunKey((current) => current + 1);
    setRun(true);
  }, []);

  const startTour = useCallback(
    async (nextTour: TourId = "main") => {
      if (nextTour === "main" && isSettingsWindow) {
        localStorage.removeItem(TOUR_COMPLETED_KEY);
        localStorage.setItem(RESTART_MAIN_TOUR_KEY, String(Date.now()));

        try {
          await commands.showWindow("main");
        } catch (error) {
          console.error("Failed to show main window for guided tour", error);
        }

        return;
      }

      runTour(nextTour);
    },
    [isSettingsWindow, runTour],
  );

  const openSettingsTour = useCallback(async () => {
    localStorage.setItem(PENDING_SETTINGS_TOUR_KEY, "true");

    try {
      await commands.showWindow("settings");
    } catch (error) {
      localStorage.removeItem(PENDING_SETTINGS_TOUR_KEY);
      console.error("Failed to show settings window for guided tour", error);
    }
  }, []);

  useEffect(() => {
    const notifyWindowSignal = () => setWindowSignal((current) => current + 1);

    const handleStorage = (event: StorageEvent) => {
      if (
        event.key === PENDING_SETTINGS_TOUR_KEY ||
        event.key === RESTART_MAIN_TOUR_KEY
      ) {
        notifyWindowSignal();
      }
    };

    const handleVisibilityChange = () => {
      if (!document.hidden) notifyWindowSignal();
    };

    window.addEventListener("storage", handleStorage);
    window.addEventListener("focus", notifyWindowSignal);
    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      window.removeEventListener("storage", handleStorage);
      window.removeEventListener("focus", notifyWindowSignal);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  useEffect(() => {
    if (run) return;
    if (isSettingsWindow) return;

    const restartId = window.setInterval(() => {
      if (!localStorage.getItem(RESTART_MAIN_TOUR_KEY)) return;

      localStorage.removeItem(RESTART_MAIN_TOUR_KEY);
      localStorage.removeItem(TOUR_COMPLETED_KEY);
      window.clearInterval(restartId);
      runTour("main");
    }, 300);

    return () => window.clearInterval(restartId);
  }, [isSettingsWindow, run, runTour, windowSignal]);

  useEffect(() => {
    if (run) return;
    if (isSettingsWindow) return;
    if (localStorage.getItem(TOUR_COMPLETED_KEY) === "true") return;

    const id = window.setInterval(() => {
      const hasStartupPermissionDialog = document.querySelector(
        STARTUP_PERMISSION_DIALOG_SELECTOR,
      );
      const targetReady = document.querySelector(
        '[data-tour="dashboard.summary"]',
      );

      if (hasStartupPermissionDialog || !targetReady) return;

      window.clearInterval(id);
      runTour("main");
    }, 300);

    return () => window.clearInterval(id);
  }, [isSettingsWindow, location.pathname, run, runTour]);

  useEffect(() => {
    if (run) return;
    if (!isSettingsWindow) return;
    if (localStorage.getItem(PENDING_SETTINGS_TOUR_KEY) !== "true") return;

    if (location.pathname !== "/settings/general") {
      navigate("/settings/general", { replace: true });
      return;
    }

    const id = window.setInterval(() => {
      const targetReady = document.querySelector(
        '[data-tour="settings.afkSensitivity"]',
      );

      if (!targetReady) return;

      localStorage.removeItem(PENDING_SETTINGS_TOUR_KEY);
      window.clearInterval(id);
      runTour("settings");
    }, 300);

    return () => window.clearInterval(id);
  }, [
    isSettingsWindow,
    location.pathname,
    navigate,
    run,
    runTour,
    windowSignal,
  ]);

  const onEvent = (data: EventData) => {
    if (data.status === STATUS.FINISHED || data.status === STATUS.SKIPPED) {
      setRun(false);

      if (tourId === "main" && data.status === STATUS.FINISHED) {
        void openSettingsTour();
      }

      if (
        tourId === "settings" ||
        (tourId === "main" && data.status === STATUS.SKIPPED)
      ) {
        localStorage.setItem(TOUR_COMPLETED_KEY, "true");
      }
    }

    if (data.action === ACTIONS.CLOSE) {
      setRun(false);
    }

    if (data.type === EVENTS.TARGET_NOT_FOUND) {
      console.warn("Tour target not found:", data.step.target);
    }
  };

  const value = useMemo(() => ({ startTour }), [startTour]);

  return (
    <TourContext.Provider value={value}>
      {children}

      <Joyride
        key={`${tourId}-${runKey}`}
        continuous
        floatingOptions={floatingOptions}
        run={run}
        steps={steps}
        onEvent={onEvent}
        scrollToFirstStep
        options={{
          zIndex: 1000,
          arrowColor: "var(--background)",
          arrowBase: 24,
          arrowSize: 12,
          arrowSpacing: 8,
          backgroundColor: "var(--background)",
          buttons: ["back", "skip", "primary"],
          closeButtonAction: "skip",
          dismissKeyAction: "close",
          offset: 4,
          overlayColor: "rgba(0, 0, 0, 0.55)",
          primaryColor: "var(--primary)",
          skipBeacon: true,
          spotlightPadding: 8,
          spotlightRadius: 8,
          showProgress: true,
          targetWaitTimeout: 6000,
          textColor: "var(--foreground)",
          width: "min(340px, calc(100vw - 32px))",
        }}
        styles={{
          floater: {
            maxWidth: "calc(100vw - 32px)",
          },
          tooltip: {
            maxHeight: "calc(100vh - 96px)",
            maxWidth: "calc(100vw - 32px)",
            overflowY: "auto",
          },
        }}
      />
    </TourContext.Provider>
  );
}
