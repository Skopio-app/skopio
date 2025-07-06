import { Button, ChipSelector, Input, Label, Switch } from "@skopio/ui";
import * as Dialog from "@radix-ui/react-dialog";
import { X } from "lucide-react";
import { useEffect, useState } from "react";
import { z } from "zod";
import { commands, GoalInput } from "../../types/tauri.gen";

enum TimeUnit {
  Hrs = "hrs",
  Mins = "mins",
  Secs = "secs",
}

enum TimeSpan {
  Day = "day",
  Week = "week",
  Month = "month",
  Year = "year",
}

enum Category {
  Coding = "coding",
  Designing = "designing",
  Debugging = "debugging",
}

const appsList = ["VSCode", "Figma", "Xcode", "Notion"];

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
  const [timeSpan, setTimeSpan] = useState<TimeSpan>(TimeSpan.Day);
  const [excludedDays, setExcludedDays] = useState<string[]>([
    "saturday",
    "sunday",
  ]);
  const [rawValue, setRawValue] = useState(
    convertFromHours(goalValue, timeUnit),
  );
  const [error, setError] = useState<string | null>(null);
  const [categories, setCategories] = useState<Category[]>([Category.Coding]);
  const [apps, setApps] = useState<string[]>([]);
  const [useCategories, setUseCategories] = useState(true);
  const [useApps, setUseApps] = useState(false);

  const categoryOptions = Object.values(Category);
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
      [TimeSpan.Day]: 24,
      [TimeSpan.Week]: 24 * 7,
      [TimeSpan.Month]: 24 * 30,
      [TimeSpan.Year]: 24 * 365,
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
    if (timeSpan === TimeSpan.Day && excludedDays.length === 7) {
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
      apps,
      categories,
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
    (timeSpan === TimeSpan.Day && excludedDays.length
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
              <ChipSelector
                values={useApps ? apps : categories}
                options={useApps ? appsList : categoryOptions}
                onToggle={(option) => {
                  if (useApps) toggleValue(setApps, option);
                  else toggleValue(setCategories, option as Category);
                }}
                onRemove={(item) => {
                  if (useApps)
                    setApps((prev) => prev.filter((a) => a !== item));
                  else
                    setCategories((prev) =>
                      prev.filter((c) => c !== (item as Category)),
                    );
                }}
              />
              <span>per</span>
              <Button
                size="sm"
                variant="secondary"
                onClick={() =>
                  setTimeSpan(cycleEnum(Object.values(TimeSpan), timeSpan))
                }
              >
                {timeSpan}
              </Button>
            </div>

            {error && <p className="text-red-500 text-sm">{error}</p>}

            {timeSpan === TimeSpan.Day && (
              <div>
                <p className="mb-2 font-medium">except for</p>
                <ChipSelector
                  options={dayOptions}
                  values={excludedDays}
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
