import * as DropDownMenu from "@radix-ui/react-dropdown-menu";
import { Check, X } from "lucide-react";
import { cn } from "../utils/cn";

type KeyLike = string | number;

interface ChipSelectorProps<T> {
  values: T[];
  options: T[];
  /** Unique, stable key for each item */
  getKey: (opt: T) => KeyLike;
  /** Render how each selected chip looks */
  renderChip: (val: T) => React.ReactNode;
  /** Render how each option in the dropdown looks */
  renderOption: (val: T) => React.ReactNode;
  /** Called when an option is picked */
  onToggle: (val: T) => void;
  onRemove: (val: T) => void;
  placeholder?: React.ReactNode;
  className?: string;
  menuClassName?: string;
  itemClassName?: string;
}

export function ChipSelector<T>({
  values,
  options,
  getKey,
  renderChip,
  renderOption,
  onToggle,
  onRemove,
  placeholder = (
    <span className="text-sm text-muted-foregorund">Select...</span>
  ),
  className,
  menuClassName,
  itemClassName,
}: ChipSelectorProps<T>) {
  const selectedKeys = new Set<KeyLike>(values.map(getKey));
  return (
    <DropDownMenu.Root modal={false}>
      <DropDownMenu.Trigger asChild>
        <div
          className={cn(
            "flex flex-wrap items-center gap-1 px-3 py-1 border rounded cursor-pointer max-w-full",
            className,
          )}
        >
          {values.length === 0
            ? placeholder
            : values.map((item) => {
                const k = getKey(item);
                return (
                  <span
                    key={k}
                    className="group flex items-center gap-1 rounded bg-accent/60 px-2 py-1 text-sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      onRemove(item);
                    }}
                  >
                    <span className="flex items-center gap-1">
                      {renderChip(item)}
                    </span>
                    <X className="h-3 w-3 opacity-70" aria-label="Remove" />
                  </span>
                );
              })}
        </div>
      </DropDownMenu.Trigger>

      <DropDownMenu.Content
        className={cn(
          "z-50 mt-1 w-64 max-h-96 overflow-y-auto rounded border bg-popover p-1 shadow",
          menuClassName,
        )}
      >
        {options.map((option) => {
          const k = getKey(option);
          const checked = selectedKeys.has(k);
          return (
            <DropDownMenu.CheckboxItem
              key={k}
              checked={checked}
              onCheckedChange={() => onToggle(option)}
              className={cn(
                "relative flex cursor-pointer select-non items-center gap-2 rounded px-2 py-2 outline-none hover:bg-accent focus:bg-accent",
                itemClassName,
              )}
            >
              <span className="flex min-w-0 items-center gap-2">
                {renderOption(option)}
              </span>

              <DropDownMenu.ItemIndicator className="ml-auto">
                <Check className="h-4 w-4" aria-hidden />
              </DropDownMenu.ItemIndicator>
            </DropDownMenu.CheckboxItem>
          );
        })}
      </DropDownMenu.Content>
    </DropDownMenu.Root>
  );
}
