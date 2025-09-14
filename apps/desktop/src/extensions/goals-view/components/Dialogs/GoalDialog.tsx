import { Button, ChipSelector, Input, Label, Switch } from "@skopio/ui";
import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { useEffect, useState } from "react";
import z from "zod/v4";
import { FieldErrors, useForm } from "react-hook-form";
import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import {
  App,
  Category,
  commands,
  Goal,
  GoalInput,
  GoalUpdateInput,
  TimeSpan,
} from "@/types/tauri.gen";
import { useGoalStore } from "../../stores/useGoalStore";
import { toast } from "sonner";

enum TimeUnit {
  Hrs = "hrs",
  Mins = "mins",
  Secs = "secs",
}

const daySchema = z.enum([
  "monday",
  "tuesday",
  "wednesday",
  "thursday",
  "friday",
  "saturday",
  "sunday",
]);

export type Day = z.infer<typeof daySchema>;

const goalFormSchema = z
  .object({
    hours: z
      .number({ error: "Enter a valid number" })
      .positive("Time must be greater than 0"),
    timeSpan: z.custom<TimeSpan>((val) =>
      ["day", "week", "month", "year"].includes(val as string),
    ),
    timeUnit: z.enum(TimeUnit),
    useApps: z.boolean(),
    useCategories: z.boolean(),
    apps: z.array(z.string().optional()),
    categories: z.array(z.string().optional()),
    excludedDays: z.array(daySchema).optional(),
  })
  .check((ctx) => {
    const maxBySpan = {
      day: 24,
      week: 168,
      month: 720,
      year: 8760,
    };

    const hoursAsHrs = convertToHours(ctx.value.hours, ctx.value.timeUnit);

    if (hoursAsHrs > maxBySpan[ctx.value.timeSpan]) {
      ctx.issues.push({
        path: ["hours"],
        code: "too_big",
        maximum: maxBySpan[ctx.value.timeSpan],
        origin: "number",
        inclusive: true,
        message: `Goal can't exceed ${maxBySpan[ctx.value.timeSpan]} hours per ${ctx.value.timeSpan}`,
        input: ctx.value.hours,
      });
    }
    if (ctx.value.useApps && (!ctx.value.apps || ctx.value.apps.length === 0)) {
      ctx.issues.push({
        path: ["apps"],
        code: "custom",
        message: "Select at least one app",
        input: ctx.value.useApps,
      });
    }
    if (
      ctx.value.useCategories &&
      (!ctx.value.categories || ctx.value.categories.length === 0)
    ) {
      ctx.issues.push({
        path: ["categories"],
        code: "custom",
        message: "Select at least one category",
        input: ctx.value.useCategories,
      });
    }
    if (ctx.value.timeSpan === "day" && ctx.value.excludedDays?.length === 7) {
      ctx.issues.push({
        path: ["excludedDays"],
        code: "custom",
        message: "You can't exclude all days",
        input: ctx.value.timeSpan,
      });
    }
  });

const TIME_SPANS: TimeSpan[] = ["day", "week", "month", "year"];

const convertToHours = (value: number, unit: TimeUnit): number => {
  switch (unit) {
    case TimeUnit.Hrs:
      return value;
    case TimeUnit.Mins:
      return value / 60;
    case TimeUnit.Secs:
      return value / 3600;
  }
};

const cycleEnum = <T,>(values: T[], current: T): T => {
  const index = values.indexOf(current);
  return values[(index + 1) % values.length];
};

interface GoalDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  goal?: Goal;
}

const GoalDialog: React.FC<GoalDialogProps> = ({
  open,
  onOpenChange,
  goal,
}) => {
  const { addGoal, updateGoal } = useGoalStore();
  const [allCategories, setAllCategories] = useState<Category[]>([]);
  const [allApps, setAllApps] = useState<App[]>([]);
  const [formError, setFormError] = useState<string | null>(null);

  const form = useForm<z.infer<typeof goalFormSchema>>({
    resolver: standardSchemaResolver(goalFormSchema),
    defaultValues: {
      hours: goal ? goal?.targetSeconds / 3600 : 2,
      timeSpan: goal?.timeSpan ?? "day",
      timeUnit: TimeUnit.Hrs,
      useApps: goal?.useApps ?? false,
      useCategories: goal?.useCategories ?? true,
      apps: goal?.apps ?? [],
      categories: goal?.categories ?? [],
      excludedDays: (goal?.excludedDays ?? [
        "saturday",
        "sunday",
      ]) as (typeof dayOptions)[number][],
    },
  });

  const { register, handleSubmit, watch, setValue } = form;

  useEffect(() => {
    const fetch = async () => {
      try {
        const apps = await commands.fetchApps();
        const categories = await commands.fetchCategories();

        setAllApps(apps);
        setAllCategories(categories);
      } catch (err) {
        console.error("Failed to fetch apps or categories:", err);
      }
    };

    fetch();
  }, []);

  const onSubmit = async (data: z.infer<typeof goalFormSchema>) => {
    const targetSeconds = convertToHours(data.hours, data.timeUnit) * 3600;

    const input: GoalInput = {
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      name: summaryText,
      targetSeconds: targetSeconds,
      timeSpan: data.timeSpan,
      useApps: data.useApps,
      useCategories: data.useCategories,
      ignoreNoActivityDays: true,
      apps: (data.apps || []).filter((a): a is string => typeof a === "string"),
      categories: (data.categories || []).filter(
        (c): c is string => typeof c === "string",
      ),
      excludedDays: data.excludedDays || [],
    };

    const updatedInput: GoalUpdateInput = {
      targetSeconds: targetSeconds,
      timeSpan: data.timeSpan,
      useApps: data.useApps,
      useCategories: data.useCategories,
      apps: (data.apps || []).filter((a): a is string => typeof a === "string"),
      categories: (data.categories || []).filter(
        (c): c is string => typeof c === "string",
      ),
      excludedDays: data.excludedDays || [],
    };

    let success = false;
    if (goal) {
      success = await updateGoal(goal.id, updatedInput);
    } else {
      success = await addGoal(input);
    }
    if (success) onOpenChange(false);
  };

  const onInvalid = (errors: FieldErrors<typeof goalFormSchema>) => {
    const firstError = Object.values(errors)[0];
    if (firstError && "message" in firstError) {
      toast.error(firstError.message as string);
    } else {
      toast.error("Please fix the highlighted errors.");
    }
  };

  useEffect(() => {
    const subscription = watch(() => {
      if (formError) setFormError(null);
    });
    return () => subscription.unsubscribe();
  }, [watch, formError]);

  const selectedApps = watch("apps");
  const selectedCategories = watch("categories");
  const useApps = watch("useApps");
  const useCategories = watch("useCategories");
  const timeSpan = watch("timeSpan");
  const excludedDays = watch("excludedDays") || [];
  const hours = watch("hours");
  const timeUnit = watch("timeUnit");

  const dayOptions = daySchema.options;

  const subject = useApps
    ? selectedApps.join(", ")
    : selectedCategories.join(", ");

  const summaryText =
    `I want to achieve ${hours} ${timeUnit}${
      subject ? ` in ${subject}` : ""
    } per ${timeSpan}` +
    (timeSpan === "day" && excludedDays.length
      ? `, except for ${excludedDays.join(", ")}`
      : "");

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-w-xl w-[90vw] max-h-[90vh] h-[600px] overflow-y-auto -translate-x-1/2 -translate-y-1/2 bg-white p-4 rounded-md shadow-lg focus:outline-none z-60">
          <div
            data-slot="dialog-header"
            className="flex justify-between items-start mb-2"
          >
            <Dialog.Title className="text-xl font-semibold break-words">
              {goal ? (
                // TODO: Add edit goal text
                <div className="space-y-1">
                  <p className="text-gray-600 text-base">{goal.name}</p>
                  <p className="text-black font-light">{summaryText}</p>
                </div>
              ) : (
                summaryText
              )}
            </Dialog.Title>
            <Dialog.Description className="sr-only">
              Set a goal for daily weekly, monthly or yearly usage of specific
              apps or categories.
            </Dialog.Description>
            <Dialog.Close asChild>
              <Button
                variant="ghost"
                type="button"
                className="text-gray-500 hover:text-black"
                aria-label="Close"
              >
                <X className="w-5 h-5" />
              </Button>
            </Dialog.Close>
          </div>

          <form
            onSubmit={handleSubmit(onSubmit, onInvalid)}
            className="space-y-6 py-4 overflow-hidden"
          >
            <div className="text-lg font-medium flex flex-wrap gap-2 items-center">
              <span>I want to achieve</span>
              <Input
                type="number"
                step={0.01}
                inputMode="decimal"
                {...register("hours", {
                  valueAsNumber: true,
                  onChange: (e) => {
                    setValue("hours", Number(e.target.value), {
                      shouldValidate: true,
                      shouldDirty: true,
                    });
                  },
                })}
                className="w-24 text-lg text-center"
              />
              <Button
                size="sm"
                type="button"
                variant="secondary"
                onClick={() => {
                  const current = watch("timeUnit");
                  const next = cycleEnum(Object.values(TimeUnit), current);
                  setValue("timeUnit", next, { shouldValidate: true });
                }}
              >
                {watch("timeUnit")}
              </Button>
              <span>in</span>
              {(useApps || useCategories) && (
                <ChipSelector<App | Category>
                  values={
                    useApps
                      ? allApps.filter((app) => selectedApps.includes(app.name))
                      : allCategories.filter((cat) =>
                          selectedCategories.includes(cat.name),
                        )
                  }
                  options={useApps ? allApps : allCategories}
                  getValueKey={(item) => item.id}
                  getOptionKey={(item) => item.id}
                  renderChip={(item) => (
                    <span className="flex items-center gap-1">
                      <span className="truncate max-w-[10rem]">
                        {item.name}
                      </span>
                    </span>
                  )}
                  renderOption={(item) => (
                    <div className="flex items-center gap-2">
                      <span className="truncate">{item.name}</span>
                    </div>
                  )}
                  onToggle={(option) => {
                    const field = useApps ? "apps" : "categories";
                    const selected = useApps
                      ? selectedApps
                      : selectedCategories;
                    if (!selected.includes(option.name)) {
                      setValue(
                        field as "apps" | "categories",
                        [...selected, option.name],
                        {
                          shouldValidate: true,
                          shouldDirty: true,
                        },
                      );
                    }
                  }}
                  onRemove={(item) => {
                    const field = useApps ? "apps" : "categories";
                    const selected = useApps
                      ? selectedApps
                      : selectedCategories;
                    setValue(
                      field as "apps" | "categories",
                      selected.filter((name) => name !== item.name),
                      { shouldValidate: true, shouldDirty: true },
                    );
                  }}
                />
              )}
              <span>per</span>
              <Button
                size="sm"
                type="button"
                variant="secondary"
                onClick={() => {
                  const next = cycleEnum(TIME_SPANS, timeSpan);
                  setValue("timeSpan", next, { shouldValidate: true });
                }}
              >
                {timeSpan}
              </Button>
            </div>

            {timeSpan === "day" && (
              <div>
                <p className="mb-2 font-medium">except for</p>
                <ChipSelector<Day, Day>
                  options={dayOptions}
                  values={excludedDays}
                  getValueKey={(d) => d}
                  getOptionKey={(d) => d}
                  renderChip={(d) => <span className="capitalize">{d}</span>}
                  renderOption={(d) => <span className="capitalize">{d}</span>}
                  onToggle={(value) =>
                    setValue(
                      "excludedDays",
                      excludedDays.includes(value)
                        ? excludedDays.filter((day) => day !== value)
                        : [...excludedDays, value],
                      { shouldValidate: true, shouldDirty: true },
                    )
                  }
                  onRemove={(value) =>
                    setValue(
                      "excludedDays",
                      excludedDays.filter((day) => day !== value),
                      { shouldValidate: true, shouldDirty: true },
                    )
                  }
                />
              </div>
            )}

            <div className="flex flex-col items-start space-y-4">
              <div className="flex flex-row space-x-2">
                <Switch
                  id="categories"
                  checked={useCategories}
                  onCheckedChange={(checked) => {
                    setValue("useCategories", checked);
                    if (checked) {
                      setValue("useApps", false);
                    } else {
                      setValue("categories", [], { shouldValidate: true });
                    }
                  }}
                />
                <Label id="categories">Filter Categories</Label>
              </div>
              <div className="flex flex-row space-x-2">
                <Switch
                  id="apps"
                  checked={useApps}
                  onCheckedChange={(checked) => {
                    setValue("useApps", checked);
                    if (checked) {
                      setValue("useCategories", false);
                    } else {
                      setValue("apps", [], { shouldValidate: true });
                    }
                  }}
                />
                <Label id="apps">Filter Apps</Label>
              </div>
            </div>
            <div className="mt-4 flex justify-center">
              <Button variant="secondary" className="w-56" type="submit">
                Save
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default GoalDialog;
