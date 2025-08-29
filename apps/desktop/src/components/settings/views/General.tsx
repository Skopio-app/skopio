import { zodResolver } from "@hookform/resolvers/zod";
import {
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Separator,
  Switch,
  Form,
  ChipSelector,
} from "@skopio/ui";
import { useEffect, useRef } from "react";
import { useForm, useWatch } from "react-hook-form";
import z from "zod/v4";
import HotkeyField from "../HotkeyField";
import { AFK, AFK_KEYS, AFK_SECONDS } from "../../../utils/constants";
import { useAutostart } from "../../../hooks/useAutostart";
import { useGlobalShortcut } from "../../../hooks/useGlobalShortcut";
import { commands, OpenApp, TrackedApp } from "../../../types/tauri.gen";
import { useOpenApps } from "../../../hooks/useOpenApps";

const TrackedAppSchema = z.object({
  name: z.string(),
  bundleId: z.string(),
});

const settingsSchema = z.object({
  launchOnStartup: z.boolean().default(false),
  globalShortcut: z
    .string()
    .min(1, "Please set a shortcut")
    .max(64)
    .refine((s) => /\+/.test(s), {
      message: "Shortcut must be a key combination (e.g., Ctrl+Shift+S)",
    }),
  afkSensitivity: z.enum(AFK_KEYS).default("1m"),
  trackedApps: z.array(TrackedAppSchema).default([]),
});

type GeneralSettingsValues = z.infer<typeof settingsSchema>;

const General = () => {
  const {
    enabled: autoEnabled,
    loading: autoLoading,
    setAutostart,
  } = useAutostart();

  const {
    shortcut,
    saveShorcut,
    loading: hotkeyLoading,
    busy,
    // error: hotkeyError,
  } = useGlobalShortcut();

  const { apps: openApps } = useOpenApps();

  const form = useForm({
    resolver: zodResolver(settingsSchema),
    defaultValues: {
      launchOnStartup: false,
      globalShortcut: shortcut,
      afkSensitivity: "1m",
      trackedApps: [],
    },
    mode: "onChange",
    reValidateMode: "onChange",
  });

  const hydratedRef = useRef(false);
  const saveTimer = useRef<number | null>(null);
  const shortcutTimer = useRef<number | null>(null);
  const afkTimer = useRef<number | null>(null);
  const trackedTimer = useRef<number | null>(null);

  const lastAfk = useRef<string>("");
  const lastSaved = useRef<boolean>(false);
  const lastShortcut = useRef<string>("");
  const lastTrackedJSON = useRef<string>("[]");

  useEffect(() => {
    const ready = !autoLoading && !hotkeyLoading;
    if (!ready || hydratedRef.current) return;

    (async () => {
      try {
        const tracked = (await commands.getConfig()).trackedApps ?? [];
        const initialAfk = form.getValues("afkSensitivity") || "1m";

        form.reset(
          {
            launchOnStartup: autoEnabled,
            globalShortcut: shortcut || "",
            afkSensitivity: initialAfk,
            trackedApps: tracked,
          },
          { keepDirty: false, keepTouched: true },
        );

        lastShortcut.current = shortcut || "";
        lastSaved.current = autoEnabled;
        lastAfk.current = initialAfk;
        lastTrackedJSON.current = JSON.stringify([...tracked].sort());
        hydratedRef.current = true;
      } catch (e) {
        console.error("Hydration failed: ", e);
      }
    })();
  }, [autoLoading, hotkeyLoading, autoEnabled, shortcut, form]);

  useEffect(() => {
    const sub = form.watch((values, info) => {
      const v = values as GeneralSettingsValues;

      if (saveTimer.current) window.clearTimeout(saveTimer.current);
      saveTimer.current = window.setTimeout(async () => {
        const ok = await form.trigger("launchOnStartup", {
          shouldFocus: false,
        });
        if (!ok) return;
        const snap = v.launchOnStartup;
        if (snap === lastSaved.current) return;
        lastSaved.current = snap;

        form.reset(v, {
          keepValues: true,
          keepDirty: false,
          keepTouched: true,
        });
      }, 300) as number;

      if (info.name === "globalShortcut") {
        const next = v.globalShortcut;
        if (next !== lastShortcut.current) {
          if (shortcutTimer.current) window.clearTimeout(shortcutTimer.current);
          shortcutTimer.current = window.setTimeout(async () => {
            const ok = await form.trigger("globalShortcut", {
              shouldFocus: false,
            });
            if (!ok) return;
            saveShorcut(next);
            lastShortcut.current = next;
          }, 250) as number;
        }
      }

      if (info.name === "afkSensitivity") {
        const nextKey = v.afkSensitivity;
        if (nextKey !== lastAfk.current) {
          if (afkTimer.current) window.clearTimeout(afkTimer.current);
          afkTimer.current = window.setTimeout(async () => {
            const ok = await form.trigger("afkSensitivity", {
              shouldFocus: false,
            });
            if (!ok) return;

            const seconds = AFK_SECONDS[nextKey];
            try {
              await commands.setAfkTimeout(seconds);
              lastAfk.current = nextKey;
            } catch (e) {
              console.error("Failed to set AFK timeout: ", e);
            }
          }, 250) as number;
        }
      }

      if (info.name === "trackedApps") {
        const nextList = v.trackedApps;
        const nextSnap = JSON.stringify([...nextList].sort());
        if (nextSnap !== lastTrackedJSON.current) {
          if (trackedTimer.current) window.clearTimeout(trackedTimer.current);
          trackedTimer.current = window.setTimeout(async () => {
            const ok = await form.trigger("trackedApps", {
              shouldFocus: false,
            });
            if (!ok) return;

            try {
              await commands.setTrackedApps(nextList);
              lastTrackedJSON.current = nextSnap;
            } catch (e) {
              console.error("Failed to save tracked apps: ", e);
            }
          }, 250) as number;
        }
      }
    });

    return () => {
      sub.unsubscribe();
      if (saveTimer.current) window.clearTimeout(saveTimer.current);
      if (shortcutTimer.current) window.clearTimeout(shortcutTimer.current);
      if (afkTimer.current) window.clearTimeout(afkTimer.current);
      if (trackedTimer.current) window.clearTimeout(trackedTimer.current);
    };
  }, [form, saveShorcut]);

  const watchedTracked = useWatch<GeneralSettingsValues>({
    control: form.control,
    name: "trackedApps",
  });

  return (
    <div className="mx-auto w-full max-w-2xl p-2">
      <Form {...form}>
        <form className="space-y-8">
          <FormField
            control={form.control}
            name="launchOnStartup"
            render={({ field }) => (
              <FormItem className="flex items-start justify-between gap-4">
                <div>
                  <FormLabel>Launch on startup</FormLabel>
                  <FormDescription>
                    Start Skopio automatically when you log in.
                  </FormDescription>
                </div>
                <FormControl>
                  <Switch
                    checked={field.value}
                    disabled={autoLoading}
                    onCheckedChange={async (next) => {
                      field.onChange(next);
                      const ok = await setAutostart(next);
                      if (!ok) {
                        field.onChange(!next);
                      }
                    }}
                  />
                </FormControl>
              </FormItem>
            )}
          />

          <Separator />

          <FormField
            control={form.control}
            name="globalShortcut"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Global shortcut</FormLabel>
                <FormDescription>
                  Choose a key combination to bring Skopio into view.
                </FormDescription>
                <FormControl>
                  <HotkeyField
                    value={field.value}
                    onChange={field.onChange}
                    placeholder="Click and press keys (e.g., ⌘ + Shift + S)"
                  />
                </FormControl>
                <FormMessage />
                {(hotkeyLoading || busy) && (
                  <p className="text-xs text-muted-foreground mt-1">
                    Updating shortcut…
                  </p>
                )}
              </FormItem>
            )}
          />

          <Separator />

          <FormField
            control={form.control}
            name="afkSensitivity"
            render={({ field }) => (
              <FormItem>
                <FormLabel>AFK sensitivity</FormLabel>
                <FormDescription>
                  How quickly Skopio marks you as idle when there’s no input.
                </FormDescription>
                <FormControl>
                  <Select value={field.value} onValueChange={field.onChange}>
                    <SelectTrigger className="w-[220px]">
                      <SelectValue placeholder="Select timeout" />
                    </SelectTrigger>
                    <SelectContent>
                      {AFK.map(([key, label]) => (
                        <SelectItem key={key} value={key}>
                          {label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />

          <Separator />

          <FormField
            control={form.control}
            name="trackedApps"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Tracked apps</FormLabel>
                <FormDescription>
                  Pick which currently open apps Skopio should track
                </FormDescription>
                <FormControl>
                  <ChipSelector<TrackedApp, OpenApp>
                    values={watchedTracked}
                    options={openApps}
                    getValueKey={(a) => a.bundleId}
                    getOptionKey={(o) => o.app.bundleId}
                    disabled={(o) => Boolean(o.blockReason)}
                    reason={(o) => o.blockReason}
                    renderChip={(a) => (
                      <span className="flex items-center gap-1">
                        <span className="truncate max-w-[10rem]">{a.name}</span>
                      </span>
                    )}
                    renderOption={(o) => (
                      <div className="flex items-center gap-2">
                        <span className="truncate">{o.app.name}</span>
                      </div>
                    )}
                    onToggle={(open) => {
                      if (open.blockReason) return;
                      const curr = new Map(
                        (field.value ?? []).map((t) => [t.bundleId, t]),
                      );
                      const id = open.app.bundleId;
                      if (curr.has(id)) {
                        curr.delete(id);
                      } else {
                        curr.set(id, { name: open.app.name, bundleId: id });
                      }
                      field.onChange(Array.from(curr.values()));
                    }}
                    onRemove={(app) => {
                      const next = (field.value ?? []).filter(
                        (t) => t.bundleId !== app.bundleId,
                      );
                      field.onChange(next);
                    }}
                  />
                </FormControl>
              </FormItem>
            )}
          />
        </form>
      </Form>
    </div>
  );
};

export default General;
