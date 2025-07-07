import { Button, ChipSelector, Input, Label, Switch } from "@skopio/ui";
import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { useEffect, useState } from "react";
import { z } from "zod";
import {
  App,
  Category,
  commands,
  GoalInput,
  TimeSpan,
} from "../../types/tauri.gen";

enum TimeUnit {
  Hrs = "hrs",
  Mins = "mins",
  Secs = "secs",
}

const goalSchema = z
  .number()
  .positive({ message: "Goal must be greater than 0" });
const daySchema = z.enum([
  "monday",
  "tuesday",
  "wednesday",
  "thursday",
  "friday",
  "saturday",
  "sunday",
]);

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

const convertFromHours = (hours: number, unit: TimeUnit): number => {
  switch (unit) {
    case TimeUnit.Hrs:
      return hours;
    case TimeUnit.Mins:
      return hours * 60;
    case TimeUnit.Secs:
      return hours * 3600;
  }
};

const cycleEnum = <T,>(values: T[], current: T): T => {
  const index = values.indexOf(current);
  return values[(index + 1) % values.length];
};

const GoalDialog = ({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) => {
  const [goalValue, setGoalValue] = useState<number>(2);
  const [timeUnit, setTimeUnit] = useState<TimeUnit>(TimeUnit.Hrs);
  const [timeSpan, setTimeSpan] = useState<TimeSpan>("day");
  const [excludedDays, setExcludedDays] = useState<string[]>([
    "saturday",
    "sunday",
  ]);
  const [rawValue, setRawValue] = useState(
    convertFromHours(goalValue, timeUnit),
  );
  const [categoryValues, setCategoryValues] = useState<Category[]>([]);
  const [appValues, setAppValues] = useState<App[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [categories, setCategories] = useState<Category[]>([]);
  const [apps, setApps] = useState<App[]>([]);
  const [useCategories, setUseCategories] = useState(true);
  const [useApps, setUseApps] = useState(false);

  useEffect(() => {
    const fetch = async () => {
      try {
        const apps = await commands.fetchApps();
        if (apps.length > 0) {
          setAppValues(apps);
        }

        const categories = await commands.fetchCategories();
        if (categories.length > 0) {
          setCategoryValues(categories);
        }
      } catch (err) {
        console.error("Failed to fetch apps or categories:", err);
      }
    };

    fetch();
  }, []);

  const categoryOptions = Object.values(categoryValues);
  const appOptions = Object.values(appValues);
  const dayOptions = daySchema.options;

  useEffect(() => {
    setRawValue(convertFromHours(goalValue, timeUnit));
  }, [timeUnit]);

  useEffect(() => {
    if (!open) setError(null);
  }, [open]);

  const handleSave = async () => {
    const parsed = goalSchema.safeParse(rawValue);
    if (!parsed.success) {
      setError(parsed.error.issues[0].message);
      return;
    }

    const hours = convertToHours(rawValue, timeUnit);

    const maxBySpan = {
      ["day"]: 24,
      ["week"]: 24 * 7,
      ["month"]: 24 * 30,
      ["year"]: 24 * 365,
    };

    const maxAllowed = maxBySpan[timeSpan];
    if (hours > maxAllowed) {
      setError(
        `Goal can't exceed ${maxAllowed} hours per ${timeSpan.toLowerCase()}`,
      );
      return;
    }

    if (hours < 0.2) {
      setError("Goal can't be less than 15 mins");
      return;
    }

    if (useCategories && categories.length === 0) {
      setError("Please select at least one category");
    }

    if (useApps && apps.length === 0) {
      setError("Please select at least one app.");
      return;
    }
    if (timeSpan === "day" && excludedDays.length === 7) {
      setError("You can't exclude all days of the week.");
      return;
    }

    const input: GoalInput = {
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      name: summaryText,
      targetSeconds: Math.round(hours * 3600),
      timeSpan,
      useApps,
      useCategories,
      ignoreNoActivityDays: true,
      apps: apps.map((a) => a.name),
      categories: categories.map((c) => c.name),
      excludedDays,
    };

    try {
      await commands.addGoal(input);
      setError(null);
      onOpenChange(false);
    } catch (err) {
      setError(`Failed to save goal: ${err}`);
    }

    setGoalValue(hours);
    setError(null);
    onOpenChange(false);
  };

  const toggleValue = <T,>(
    setList: React.Dispatch<React.SetStateAction<T[]>>,
    value: T,
  ) => {
    setList((prev) =>
      prev.includes(value) ? prev.filter((v) => v !== value) : [...prev, value],
    );
  };

  const subject = useApps ? apps.join(", ") : categories.join(", ");

  const summaryText =
    `I want to achieve ${rawValue} ${timeUnit}` +
    `${categories.length || apps.length ? ` in ${subject}` : ""}` +
    ` per ${timeSpan}` +
    (timeSpan === "day" && excludedDays.length
      ? `, except for ${excludedDays.join(", ")}`
      : "");

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-w-xl w-[90vw] max-h-[90vh] h-[600px] overflow-y-auto -translate-x-1/2 -translate-y-1/2 bg-white p-4 rounded-md shadow-lg focus:outline-none">
          <div className="flex justify-between items-start mb-2">
            <Dialog.Title className="text-xl font-semibold break-words">
              {summaryText}
            </Dialog.Title>
            <Dialog.Close asChild>
              <button
                className="text-gray-500 hover:text-black"
                aria-label="Close"
              >
                <X className="w-5 h-5" />
              </button>
            </Dialog.Close>
          </div>

          <div className="space-y-6 py-4 overflow-hidden">
            <div className="text-lg font-medium flex flex-wrap gap-2 items-center">
              <span>I want to achieve</span>
              <Input
                type="number"
                step="0.1"
                value={rawValue}
                onChange={(e) => setRawValue(parseFloat(e.target.value))}
                className="w-24 text-lg text-center"
              />
              <Button
                size="sm"
                variant="secondary"
                onClick={() =>
                  setTimeUnit(cycleEnum(Object.values(TimeUnit), timeUnit))
                }
              >
                {timeUnit}
              </Button>
              <span>in</span>
              {(useApps || useCategories) && (
                <ChipSelector
                  values={useApps ? apps : categories}
                  options={useApps ? appOptions : categoryOptions}
                  getLabel={(item) => item.name}
                  onToggle={(option) => {
                    if (useApps) toggleValue(setApps, option as unknown as App);
                    else
                      toggleValue(setCategories, option as unknown as Category);
                  }}
                  onRemove={(item) => {
                    if (useApps)
                      setApps((prev) =>
                        prev.filter(
                          (a) => a.id !== (item as unknown as App).id,
                        ),
                      );
                    else
                      setCategories((prev) =>
                        prev.filter(
                          (c) => c.id !== (item as unknown as Category).id,
                        ),
                      );
                  }}
                />
              )}
              <span>per</span>
              <Button
                size="sm"
                variant="secondary"
                onClick={() =>
                  setTimeSpan(cycleEnum(Object.values(TIME_SPANS), timeSpan))
                }
              >
                {timeSpan}
              </Button>
            </div>

            {error && <p className="text-red-500 text-sm">{error}</p>}

            {timeSpan === "day" && (
              <div>
                <p className="mb-2 font-medium">except for</p>
                <ChipSelector
                  options={dayOptions}
                  values={excludedDays}
                  getLabel={(item) => item}
                  onToggle={(value) =>
                    setExcludedDays((prev) =>
                      prev.includes(value)
                        ? prev.filter((d) => d !== value)
                        : [...prev, value],
                    )
                  }
                  onRemove={(value) =>
                    setExcludedDays((prev) => prev.filter((d) => d !== value))
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
                    setUseCategories(checked);
                    if (checked) {
                      setUseApps(false);
                    } else {
                      setCategories([]);
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
                    setUseApps(checked);
                    if (checked) {
                      setUseCategories(false);
                    } else {
                      setApps([]);
                    }
                  }}
                />
                <Label id="apps">Filter Apps</Label>
              </div>
            </div>
          </div>

          <div className="mt-4 flex justify-center">
            <Button className="w-56" onClick={handleSave}>
              Save
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default GoalDialog;
