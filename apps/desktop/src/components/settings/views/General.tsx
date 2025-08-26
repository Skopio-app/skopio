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
import { useEffect, useMemo, useRef, useState } from "react";
import { useForm } from "react-hook-form";
import z from "zod/v4";
import HotkeyField from "../HotkeyField";
import { AFK, AFK_KEYS, AFK_SECONDS } from "../../../utils/constants";
import { useAutostart } from "../../../hooks/useAutostart";
import { useGlobalShortcut } from "../../../hooks/useGlobalShortcut";
import { commands } from "../../../types/tauri.gen";

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
  trackedApps: z.array(z.string()).default([]),
});

type GeneralSettingsValues = z.infer<typeof settingsSchema>;
type AppInfo = Awaited<ReturnType<typeof commands.getOpenApps>>[number];

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

  const [openApps, setOpenApps] = useState<AppInfo[]>([]);
  const byBundleId = useMemo(() => {
    const m = new Map<string, AppInfo>();
    for (const a of openApps) m.set(a.bundle_id, a);
    return m;
  }, [openApps]);

  const form = useForm<GeneralSettingsValues>({
    resolver: zodResolver(settingsSchema),
    defaultValues: {
      launchOnStartup: false,
      globalShortcut: shortcut,
      afkSensitivity: "1m",
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
  const lastSavedJSON = useRef<string>(JSON.stringify(form.getValues()));
  const lastShortcut = useRef<string>("");
  const lastTrackedJSON = useRef<string>("[]");

  useEffect(() => {
    const ready = !autoLoading && !hotkeyLoading;
    if (!ready || hydratedRef.current) return;

    (async () => {
      try {
        const [tracked, open] = await Promise.all([
          (await commands.getConfig()).trackedApps,
          commands.getOpenApps().catch(() => [] as AppInfo[]),
        ]);

        const initialAfk = form.getValues("afkSensitivity") || "1m";

        form.reset(
          {
            launchOnStartup: autoEnabled,
            globalShortcut: shortcut || "",
            afkSensitivity: initialAfk,
            trackedApps: tracked ?? [],
          },
          { keepDirty: false, keepTouched: true },
        );

        setOpenApps(open ?? []);

        lastShortcut.current = shortcut || "";
        lastSavedJSON.current = JSON.stringify(form.getValues());
        lastAfk.current = initialAfk;
        lastTrackedJSON.current = JSON.stringify(tracked ?? []);
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
        const ok = await form.trigger(undefined, { shouldFocus: false });
        if (!ok) return;
        const snap = JSON.stringify(v);
        if (snap === lastSavedJSON.current) return;
        lastSavedJSON.current = snap;

        form.reset(v, {
          keepValues: true,
          keepDirty: false,
          keepTouched: true,
        });
      }, 300) as number;

      if (info.name === "globalShortcut") {
        const next = v.globalShortcut || "";
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
        const nextList = v.trackedApps ?? [];
        const nextSnap = JSON.stringify(nextList.slice().sort());
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

  const selectedAppInfos = useMemo(() => {
    const ids = form.getValues("trackedApps") ?? [];
    return ids.map((id) => byBundleId.get(id)).filter(Boolean) as AppInfo[];
  }, [byBundleId, form]);

  const labelFor = (a: { app_name?: string; bundle_id: string }) =>
    a.app_name?.trim() || a.bundle_id;

  const allOptions = openApps;

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
                  <ChipSelector<AppInfo>
                    values={
                      // use selected infos if we have them; otherwise render chips by IDs with fallbacks
                      selectedAppInfos.length
                        ? selectedAppInfos
                        : (field.value ?? []).map((id) => ({
                            app_name: id,
                            bundle_id: id,
                            path: "",
                            pid: 0,
                          }))
                    }
                    options={allOptions}
                    getLabel={(a) => labelFor(a)}
                    onToggle={(app) => {
                      const id = app.bundle_id;
                      const curr = new Set(field.value ?? []);
                      if (curr.has(id)) curr.delete(id);
                      else curr.add(id);
                      field.onChange(Array.from(curr));
                    }}
                    onRemove={(app) => {
                      const id = app.bundle_id;
                      const next = (field.value ?? []).filter((x) => x !== id);
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
