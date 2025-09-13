import { Button } from "@skopio/ui";
import { toast } from "sonner";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { useEffect } from "react";

type ProgressState = {
  pct: number;
  downloaded: number;
  total: number;
  phase: "idle" | "downloading" | "installing" | "finished" | "error";
  version?: string;
  notes?: string;
  date?: string;
  error?: string;
};

const formatMB = (n: number) => (n / (1024 * 1024)).toFixed(1);

const showUpdaterToast = (meta: {
  version: string;
  body?: string;
  date?: string;
}) => {
  const id = `update-${meta.version}`;
  const state: ProgressState = {
    pct: 0,
    downloaded: 0,
    total: 0,
    phase: "idle",
    version: meta.version,
    notes: meta.body,
    date: meta.date,
  };

  let onDownload: (() => void) | undefined;

  const render = () =>
    toast.custom(
      () => (
        <div className="w-[28rem] max-w-[90vw] rounded-xl border bg-neutral-50 p-3 shadow-lg">
          <div className="flex items-start gap-2">
            <div className="flex-1">
              <p className="text-sm font-semibold text-neutral-900">
                Update available - {state.version}
              </p>
              {state.notes && (
                <p className="mt-1 line-clamp-3 whitespace-pre-line text-xs text-neutral-600">
                  {state.notes}
                </p>
              )}
            </div>
          </div>

          {state.phase !== "idle" && (
            <div className="mt-3 space-y-1.5">
              <div className="h-2 w-full overflow-hidden rounded-full bg-neutral-200">
                <div
                  className="h-full rounded-full bg-neutral-900 transition-[width]"
                  style={{ width: `${state.pct}%` }}
                />
              </div>
              <div className="flex items-center justify-between text-xs text-neutral-600">
                <span className="animate-pulse">
                  {state.phase === "downloading" && "Downloading..."}
                  {state.phase === "installing" && "Installing..."}
                  {state.phase === "finished" && "Ready to relaunch"}
                  {state.phase === "error" && "Download failed"}
                </span>
                <span>
                  {state.total > 0
                    ? `${formatMB(state.downloaded)} / ${formatMB(state.total)} MB (${Math.floor(
                        state.pct,
                      )}%)`
                    : state.phase === "installing"
                      ? "Finalizing..."
                      : ""}
                </span>
              </div>
            </div>
          )}

          <div className="mt-3 flex items-center justify-end gap-2">
            {state.phase === "idle" && (
              <>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => toast.dismiss(id)}
                  className="text-neutral-600"
                >
                  Later
                </Button>
                <Button size="sm" onClick={() => onDownload?.()}>
                  Download
                </Button>
              </>
            )}

            {state.phase === "finished" && (
              <>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => toast.dismiss(id)}
                  className="text-neutral-600"
                >
                  Close
                </Button>
                <Button
                  variant="secondary"
                  size="sm"
                  onClick={() => relaunch()}
                >
                  Relaunch
                </Button>
              </>
            )}

            {state.phase === "error" && (
              <>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => toast.dismiss(id)}
                  className="text-neutral-600"
                >
                  Dismiss
                </Button>
                <Button size="sm" onClick={() => onDownload?.()}>
                  Retry
                </Button>
              </>
            )}
          </div>

          {state.phase === "error" && state.error && (
            <p className="mt-2 text-xs text-red-600">{state.error}</p>
          )}
        </div>
      ),
      { id, duration: Infinity },
    );

  const rerender = () => render();

  const api = {
    start(total: number) {
      state.phase = "downloading";
      state.total = total || 0;
      state.pct = total ? (state.downloaded / total) * 100 : 0;
      rerender();
    },
    progress(chunk: number) {
      state.downloaded += chunk;
      state.pct = state.total
        ? (state.downloaded / state.total) * 100
        : state.pct;
      rerender();
    },
    installing() {
      state.phase = "installing";
      rerender();
    },
    finished() {
      state.phase = "finished";
      state.pct = 100;
      rerender();
    },
    error(msg?: string) {
      state.phase = "error";
      state.error = msg || "Something went wrong.";
      rerender();
    },
    setOnDownload(fn: () => void) {
      onDownload = fn;
      rerender();
    },
    showInitial() {
      state.phase = "idle";
      render();
    },
    dismiss() {
      toast.dismiss(id);
    },
  };

  return api;
};

const runUpdaterFlow = async () => {
  const update = await check().catch((e) => console.warn("Updater error: ", e));
  if (!update) return;

  const ui = showUpdaterToast({
    version: update.version,
    body: update.body,
    date: update.date,
  });

  ui.setOnDownload(async () => {
    try {
      let total = 0;

      await update.downloadAndInstall((ev) => {
        switch (ev.event) {
          case "Started":
            total = ev.data.contentLength || 0;
            ui.start(total);
            break;
          case "Progress":
            ui.progress(ev.data.chunkLength);
            break;
          case "Finished":
            ui.installing();
            break;
        }
      });

      ui.finished();
    } catch (e: any) {
      ui.error(typeof e?.message === "string" ? e.message : String(e));
    }
  });

  ui.showInitial();
};

const UpdaterToast = () => {
  useEffect(() => {
    void runUpdaterFlow();
  }, []);
  return null;
};

export default UpdaterToast;
