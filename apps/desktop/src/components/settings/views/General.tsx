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
import { AFK, AFK_KEYS, AFK_SECONDS } from "../../../utils/constants";

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

async function saveSettings(values: GeneralSettingsValues) {
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
    const sub = form.watch((values) => {
      latestValuesRef.current = values as GeneralSettingsValues;

      if (timerRef.current) window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(async () => {
        const valid = await form.trigger(undefined, { shouldFocus: false });
        if (!valid) return;
        const snapshot = JSON.stringify(latestValuesRef.current);
        if (snapshot === lastSavedRef.current) return;

        await saveSettings(latestValuesRef.current);

        lastSavedRef.current = snapshot;

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
                  Choose a key combination to toggle Skopio.
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
