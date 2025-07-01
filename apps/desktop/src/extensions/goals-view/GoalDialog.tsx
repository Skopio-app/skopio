import { Button, Input, Label, Switch } from "@skopio/ui";
import * as Dialog from "@radix-ui/react-dialog";
import * as DropdownMenu from "@radix-ui/react-dropdown-menu";
import { X } from "lucide-react";
import { useEffect, useState } from "react";
import { z } from "zod";

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
  const [ignoreNoActivityDays, setIgnoreNoActivityDays] =
    useState<boolean>(false);
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

  const handleSave = () => {
    const parsed = goalSchema.safeParse(rawValue);
    if (!parsed.success) {
      setError(parsed.error.issues[0].message);
      return;
    }
    setGoalValue(convertToHours(rawValue, timeUnit));
    onOpenChange(false);
    setError(null);
  };

  const toggleValue = <T,>(
    list: T[],
    setList: React.Dispatch<React.SetStateAction<T[]>>,
    value: T,
  ) => {
    setList((prev) =>
      prev.includes(value) ? prev.filter((v) => v !== value) : [...prev, value],
    );
  };

  const subject = useApps ? apps.join(", ") : categories.join(", ");

  const summaryText =
    `I want to achieve ${rawValue} ${timeUnit} in ${subject} per ${timeSpan}` +
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
            <div className="flex items-center space-x-4">
              <Label>Use Categories</Label>
              <Switch
                checked={useCategories}
                onCheckedChange={(checked) => {
                  setUseCategories(checked);
                  if (checked) setUseApps(false);
                }}
              />
              <Label>Use Apps</Label>
              <Switch
                checked={useApps}
                onCheckedChange={(checked) => {
                  setUseApps(checked);
                  if (checked) setUseCategories(false);
                }}
              />
            </div>

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

              <DropdownMenu.Root modal={false}>
                <DropdownMenu.Trigger asChild>
                  <div className="flex flex-wrap items-center gap-1 px-3 py-1 border rounded cursor-pointer max-w-full">
                    {(useApps ? apps : categories).map((item) => (
                      <span
                        key={item}
                        className="flex items-center gap-1 px-2 py-1 bg-gray-200 rounded whitespace-nowrap"
                        onClick={(e) => {
                          e.stopPropagation();
                          if (useApps) {
                            setApps((prev) => prev.filter((a) => a !== item));
                          } else {
                            setCategories((prev) =>
                              prev.filter((c) => c !== item),
                            );
                          }
                        }}
                      >
                        {item} ×
                      </span>
                    ))}
                  </div>
                </DropdownMenu.Trigger>
                <DropdownMenu.Content className="z-50 mt-1 w-48 max-h-60 overflow-y-auto bg-white border rounded shadow">
                  {(useApps ? appsList : categoryOptions).map((opt) => (
                    <DropdownMenu.Item
                      key={opt}
                      onSelect={() => {
                        if (useApps) {
                          toggleValue(apps, setApps, opt);
                        } else {
                          toggleValue(
                            categories,
                            setCategories,
                            opt as Category,
                          );
                        }
                      }}
                      className="px-3 py-2 cursor-pointer hover:bg-gray-100"
                    >
                      {opt}
                    </DropdownMenu.Item>
                  ))}
                </DropdownMenu.Content>
              </DropdownMenu.Root>

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
                <p className="mb-2 font-medium">Except for</p>
                <DropdownMenu.Root modal={false}>
                  <DropdownMenu.Trigger asChild>
                    <div className="flex flex-wrap items-center gap-1 px-3 py-1 border rounded cursor-pointer max-w-full">
                      {excludedDays.map((day) => (
                        <span
                          key={day}
                          className="flex items-center gap-1 px-2 py-1 bg-gray-200 rounded whitespace-nowrap"
                          onClick={(e) => {
                            e.stopPropagation();
                            setExcludedDays((prev) =>
                              prev.filter((d) => d !== day),
                            );
                          }}
                        >
                          {day} ×
                        </span>
                      ))}
                    </div>
                  </DropdownMenu.Trigger>
                  <DropdownMenu.Content className="z-50 mt-1 w-48 max-h-60 overflow-y-auto bg-white border rounded shadow">
                    {dayOptions.map((day) => (
                      <DropdownMenu.Item
                        key={day}
                        onSelect={() =>
                          toggleValue(excludedDays, setExcludedDays, day)
                        }
                        className="px-3 py-2 cursor-pointer hover:bg-gray-100"
                      >
                        {day}
                      </DropdownMenu.Item>
                    ))}
                  </DropdownMenu.Content>
                </DropdownMenu.Root>
              </div>
            )}

            <div className="flex items-center space-x-3">
              <Switch
                checked={ignoreNoActivityDays}
                onCheckedChange={setIgnoreNoActivityDays}
              />
              <span>Ignore days with no activity</span>
            </div>
          </div>

          <div className="mt-4 flex justify-end">
            <Button onClick={handleSave}>Save</Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default GoalDialog;
