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
import { useEffect, useMemo, useRef } from "react";
import { useForm } from "react-hook-form";
import z from "zod/v4";
import HotkeyField from "../HotkeyField";
const AFK_SECONDS = {
  "30s": 30,
  "1m": 60,
  "2m": 120,
  "3m": 180,
  "5m": 300,
  "10m": 600,
} as const;

const settingsSchema = z.object({
  launchOnStartup: z.boolean().default(false),
  globalShortcut: z
    .string()
    .min(1, "Please set a shortcut")
    .max(64)
    .refine((s) => /\+/.test(s), {
      message: "Shortcut must be a key combination (e.g., Ctrl+Shift+S)",
    }),
  afkSensitivity: z.enum(["30s", "1m", "2m", "3m", "5m", "10m"]).default("1m"),
});

type GeneralSettingsValues = z.infer<typeof settingsSchema>;

async function saveSettings(values: GeneralSettingsValues) {
  // TODO: replace with Tauri invoke / your settings store
  console.log("Auto-saved:", {
    ...values,
    afkSeconds: AFK_SECONDS[values.afkSensitivity],
  });
}

const General = () => {
  const form = useForm<GeneralSettingsValues>({
    resolver: zodResolver(settingsSchema),
    defaultValues: {
      launchOnStartup: false,
      globalShortcut: "⌘+Shift+S",
      afkSensitivity: "1m",
    },
    mode: "onChange",
    reValidateMode: "onChange",
  });

  const timerRef = useRef<number | null>(null);
  const lastSavedRef = useRef<string>(JSON.stringify(form.getValues()));
  const latestValuesRef = useRef<GeneralSettingsValues>(form.getValues());

  useEffect(() => {
    // subscribe to ALL changes; store latest values
    const sub = form.watch((values) => {
      latestValuesRef.current = values as GeneralSettingsValues;

      // debounce
      if (timerRef.current) window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(async () => {
        // Validate all fields
        const valid = await form.trigger(undefined, { shouldFocus: false });
        if (!valid) return;

        // Avoid re-saving identical values
        const snapshot = JSON.stringify(latestValuesRef.current);
        if (snapshot === lastSavedRef.current) return;

        await saveSettings(latestValuesRef.current);

        // Mark current state as the new baseline
        lastSavedRef.current = snapshot;

        // Clear dirty flags without changing what the user sees
        form.reset(latestValuesRef.current, {
          keepValues: true,
          keepDirty: false,
          keepTouched: true,
        });
      }, 300) as unknown as number;
    });

    return () => {
      if (timerRef.current) window.clearTimeout(timerRef.current);
      sub.unsubscribe();
    };
  }, [form]);

  const afkItems = useMemo(
    () =>
      [
        { key: "30s", label: "30 seconds" },
        { key: "1m", label: "1 minute" },
        { key: "2m", label: "2 minutes" },
        { key: "3m", label: "3 minutes" },
        { key: "5m", label: "5 minutes" },
        { key: "10m", label: "10 minutes" },
      ] as const,
    [],
  );

  return (
    <div className="mx-auto w-full max-w-2xl p-2">
      <h2 className="font-semibold leading-none tracking-tight pb-6">
        General Settings
      </h2>
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
                    onCheckedChange={field.onChange}
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
                  Choose a key combination to toggle Skopio or open the command
                  palette.
                </FormDescription>
                <FormControl>
                  <HotkeyField
                    value={field.value}
                    onChange={field.onChange}
                    placeholder="Click and press keys (e.g., ⌘ + Shift + S)"
                  />
                </FormControl>
                <FormMessage />
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
                      {afkItems.map(({ key, label }) => (
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
