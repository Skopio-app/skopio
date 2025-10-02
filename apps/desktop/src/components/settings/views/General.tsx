import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
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
  cn,
} from "@skopio/ui";
import { useEffect, useRef } from "react";
import { useForm } from "react-hook-form";
import * as z from "zod/v4";
import HotkeyField from "@/components/settings/HotkeyField";
import { AFK, AFK_KEYS, AFK_SECONDS, AFK_LABELS } from "@/utils/constants";
import { useAutostart } from "@/hooks/useAutostart";
import { useGlobalShortcut } from "@/hooks/useGlobalShortcut";
import { commands, OpenApp, TrackedApp, Theme } from "@/types/tauri.gen";
import { useOpenApps } from "@/hooks/useOpenApps";
import { Monitor, Moon, Sun } from "lucide-react";
import { setTheme as setTauriTheme } from "@tauri-apps/api/app";
import { useTheme } from "@/utils/theme";

const TrackedAppSchema = z.object({
  name: z.string(),
  bundleId: z.string(),
});

const THEME_OPTIONS = ["light", "dark", "system"] as Theme[];
type ThemeValue = (typeof THEME_OPTIONS)[number];

const settingsSchema = z.object({
  launchOnStartup: z.boolean(),
  globalShortcut: z
    .string()
    .min(1, "Please set a shortcut")
    .max(64)
    .refine((s) => /\+/.test(s), {
      message: "Shortcut must be a key combination (e.g., Ctrl+Shift+S)",
    }),
  afkSensitivity: z.enum(AFK_KEYS),
  trackedApps: z.array(TrackedAppSchema),
  theme: z.enum(THEME_OPTIONS),
});

type GeneralSettingsValues = z.infer<typeof settingsSchema>;

const General = () => {
  const {
    enabled: autoEnabled,
    loading: autoLoading,
    setAutostart,
  } = useAutostart();

  const { shortcut, saveShorcut, loading: hotkeyLoading } = useGlobalShortcut();
  const { apps: openApps, fetch: fetchApps } = useOpenApps();
  const { setTheme: setUITheme } = useTheme();

  const form = useForm<GeneralSettingsValues>({
    resolver: standardSchemaResolver(settingsSchema),
    defaultValues: {
      launchOnStartup: false,
      globalShortcut: shortcut,
      afkSensitivity: "1m",
      trackedApps: [],
      theme: "system",
    },
    mode: "onChange",
    reValidateMode: "onChange",
  });

  const hydratedRef = useRef(false);
  const saveTimer = useRef<number | null>(null);
  const shortcutTimer = useRef<number | null>(null);
  const afkTimer = useRef<number | null>(null);
  const trackedTimer = useRef<number | null>(null);
  const themeTimer = useRef<number | null>(null);

  const lastAfk = useRef<string>("");
  const lastSaved = useRef<boolean>(false);
  const lastShortcut = useRef<string>("");
  const lastTrackedJSON = useRef<string>("[]");
  const lastTheme = useRef<ThemeValue>("system");

  useEffect(() => {
    const ready = !autoLoading && !hotkeyLoading;
    if (!ready || hydratedRef.current) return;

    (async () => {
      try {
        const cfg = await commands.getConfig();
        const tracked = cfg.trackedApps ?? [];
        const afk = cfg.afkTimeout;
        const theme = cfg.theme;
        const seconds = AFK_LABELS[afk];

        form.reset(
          {
            launchOnStartup: autoEnabled,
            globalShortcut: shortcut || "",
            afkSensitivity: seconds,
            trackedApps: tracked,
            theme,
          },
          { keepDirty: false, keepTouched: true },
        );

        lastShortcut.current = shortcut || "";
        lastSaved.current = autoEnabled;
        lastAfk.current = seconds;
        lastTrackedJSON.current = JSON.stringify([...tracked].sort());
        lastTheme.current = theme;
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

      if (info.name === "theme") {
        const next = v.theme;
        if (next !== lastTheme.current) {
          if (themeTimer.current) window.clearTimeout(themeTimer.current);
          themeTimer.current = window.setTimeout(async () => {
            const ok = await form.trigger("theme", { shouldFocus: false });
            if (!ok) return;
            try {
              setUITheme(next);
              await setTauriTheme(next === "system" ? null : next);
              await commands.setTheme(next);
              lastTheme.current = next;
            } catch (e) {
              console.error("Failed to set theme: ", e);
            }
          }, 200) as number;
        }
      }
    });

    return () => {
      sub.unsubscribe();
      if (saveTimer.current) window.clearTimeout(saveTimer.current);
      if (shortcutTimer.current) window.clearTimeout(shortcutTimer.current);
      if (afkTimer.current) window.clearTimeout(afkTimer.current);
      if (trackedTimer.current) window.clearTimeout(trackedTimer.current);
      if (themeTimer.current) window.clearTimeout(themeTimer.current);
    };
  }, [form, saveShorcut]);

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
                {hotkeyLoading && (
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
            name="theme"
            render={({ field }) => {
              const items: {
                value: ThemeValue;
                label: string;
                Icon: React.ComponentType<any>;
              }[] = [
                { value: "light", label: "Light", Icon: Sun },
                { value: "dark", label: "Dark", Icon: Moon },
                { value: "system", label: "System", Icon: Monitor },
              ];

              return (
                <FormItem>
                  <FormLabel>Appearance</FormLabel>
                  <FormDescription>Choose Skopio's appearance.</FormDescription>
                  <FormControl>
                    <div className="flex items-stretch gap-3">
                      {items.map(({ value, label, Icon }) => {
                        const active = field.value === value;
                        return (
                          <button
                            type="button"
                            key={value}
                            onClick={() => field.onChange(value)}
                            className={cn(
                              "flex w-24 flex-col items-center justify-center rounded-xl border px-3 py-3 transition",
                              active
                                ? "border-primary ring-2 ring-primary/40 text-primary"
                                : "border-muted bg-background hover:bg-accent/40 text-muted-foreground",
                            )}
                            aria-pressed={active}
                          >
                            <Icon className="h-6 w-6" />
                            <span className="mt-1 text-xs">{label}</span>
                          </button>
                        );
                      })}
                    </div>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              );
            }}
          />

          <Separator />

          <FormField
            control={form.control}
            name="afkSensitivity"
            render={({ field }) => (
              <FormItem>
                <FormLabel>AFK sensitivity</FormLabel>
                <FormDescription>
                  How quickly Skopio marks you as idle when there's no input.
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
                  Pick which currently open apps Skopio should track.
                </FormDescription>
                <FormControl>
                  <ChipSelector<TrackedApp, OpenApp>
                    values={field.value ?? []}
                    options={openApps}
                    getValueKey={(a) => a.bundleId}
                    getOptionKey={(o) => o.app.bundleId}
                    disabled={(o) => Boolean(o.blockReason)}
                    reason={(o) => o.blockReason}
                    onOpenChange={(open) => {
                      if (open) fetchApps();
                    }}
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
