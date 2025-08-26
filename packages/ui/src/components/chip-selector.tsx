import * as DropDownMenu from "@radix-ui/react-dropdown-menu";
import { X } from "lucide-react";
import { cn } from "../utils/cn";

type KeyLike = string | number;

interface ChipSelectorProps<TValue, TOption> {
  values: TValue[];
  options: TOption[];
  /** Unique, stable key for each selected value */
  getValueKey: (val: TValue) => KeyLike;
  /** Unique, stable key for each option */
  getOptionKey: (opt: TOption) => KeyLike;
  /** Render how each selected chip looks */
  renderChip: (val: TValue) => React.ReactNode;
  /** Render how each option in the dropdown looks */
  renderOption: (val: TOption) => React.ReactNode;
  /** Called when an option is picked */
  onToggle: (val: TOption) => void;
  onRemove: (val: TValue) => void;
  placeholder?: React.ReactNode;
  className?: string;
  menuClassName?: string;
  itemClassName?: string;
}

export function ChipSelector<TValue, TOption>({
  values,
  options,
  getValueKey,
  getOptionKey,
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
}: ChipSelectorProps<TValue, TOption>) {
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
                const k = getValueKey(item);
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
          "z-50 mt-1 w-64 max-h-72 overflow-y-auto rounded border bg-popover p-1 shadow",
          menuClassName,
        )}
      >
        {options.map((option) => {
          const k = getOptionKey(option);
          return (
            <DropDownMenu.Item
              key={k}
              onSelect={() => onToggle(option)}
              className={cn(
                "cursor-pointer rounded px-2 py-2 outline-none hover:bg-accent focus:bg-accent",
                itemClassName,
              )}
            >
              {renderOption(option)}
            </DropDownMenu.Item>
          );
        })}
      </DropDownMenu.Content>
    </DropDownMenu.Root>
  );
}
