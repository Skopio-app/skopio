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
} from "@skopio/ui";
import { useEffect, useRef } from "react";
import { useForm } from "react-hook-form";
import z from "zod/v4";
import HotkeyField from "../HotkeyField";
import { AFK, AFK_KEYS } from "../../../utils/constants";
import { useAutostart } from "../../../hooks/useAutostart";
import { useGlobalShortcut } from "../../../hooks/useGlobalShortcut";

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
    setAndPersist,
    loading: hotkeyLoading,
    busy,
    error: hotkeyError,
  } = useGlobalShortcut();

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
  useEffect(() => {
    const ready = !autoLoading && !hotkeyLoading;
    if (!ready || hydratedRef.current) return;

    hydratedRef.current = true;
    form.reset(
      {
        launchOnStartup: autoEnabled,
        globalShortcut: shortcut || "",
        afkSensitivity: form.getValues("afkSensitivity") || "1m",
      },
      { keepDirty: false, keepTouched: true },
    );
  }, [autoLoading, hotkeyLoading, autoEnabled, shortcut, form]);

  const saveTimer = useRef<number | null>(null);
  const shortcutTimer = useRef<number | null>(null);
  const lastSavedJSON = useRef<string>(JSON.stringify(form.getValues()));
  const lastShortcut = useRef<string>("");

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
            await setAndPersist(next);
            lastShortcut.current = next;
          }, 250) as number;
        }
      }
    });

    return () => {
      sub.unsubscribe();
      if (saveTimer.current) window.clearTimeout(saveTimer.current);
      if (shortcutTimer.current) window.clearTimeout(shortcutTimer.current);
    };
  }, [form, setAndPersist]);

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
                {hotkeyError && <FormMessage>{hotkeyError}</FormMessage>}
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
        </form>
      </Form>
    </div>
  );
};

export default General;
