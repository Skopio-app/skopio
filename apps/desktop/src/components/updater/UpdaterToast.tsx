import { Button } from "@skopio/ui";
import { toast } from "sonner";
import { relaunch } from "@tauri-apps/plugin-process";
import { check } from "@tauri-apps/plugin-updater";
import { useEffect } from "react";
import Markdown, { Components } from "react-markdown";
import remarkGfm from "remark-gfm";

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

const mdComponents: Components = {
  h1: (p: any) => <h1 className="mt-2 text-base font-semibold" {...p} />,
  h2: (p: any) => <h2 className="mt-3 text-sm font-semibold" {...p} />,
  h3: (p: any) => <h3 className="mt-3 text-sm font-medium" {...p} />,
  p: (p: any) => (
    <p className="mt-2 text-xs leading-relaxed text-neutral-700" {...p} />
  ),
  ul: (p: any) => (
    <ul className="mt-2 ml-4 list-disc space-y-1 text-xs" {...p} />
  ),
  ol: (p: any) => (
    <ol className="mt-2 ml-4 list-decimal space-y-1 text-xs" {...p} />
  ),
  li: (p: any) => <li className="marker:text-neutral-500" {...p} />,
  a: (p: any) => (
    <a className="underline underline-offset-2 hover:opacity-80" {...p} />
  ),
  hr: () => <div className="my-3 h-px w-full bg-neutral-200" />,
  code: ({ inline, className, children, ...props }: any) =>
    inline ? (
      <code
        className="rounded bg-neutral-100 px-1 py-0.5 text-[11px]"
        {...props}
      >
        {children}
      </code>
    ) : (
      <pre
        className="mt-2 max-h-56 overflow-auto rounded bg-neutral-900 p-2 text-[11px] text-neutral-100"
        {...props}
      >
        <code className={className}>{children}</code>
      </pre>
    ),
  blockquote: (p: any) => (
    <blockquote
      className="mt-2 border-l-2 border-neutral-300 pl-2 text-xs italic text-neutral-600"
      {...p}
    />
  ),
  table: (p: any) => (
    <table className="mt-2 w-full text-xs border-collapse" {...p} />
  ),
  th: (p: any) => (
    <th className="border-b px-2 py-1 text-left font-medium" {...p} />
  ),
  td: (p: any) => <td className="border-b px-2 py-1 align-top" {...p} />,
};

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
                Update available - v{state.version}
              </p>
              {state.notes && (
                <div className="mt-1 prose prose-invert text-xs text-neutral-800 max-w-none">
                  <Markdown
                    components={mdComponents}
                    remarkPlugins={[remarkGfm]}
                  >
                    {state.notes}
                  </Markdown>
                </div>
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
