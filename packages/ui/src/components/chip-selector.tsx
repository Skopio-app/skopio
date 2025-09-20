import * as DropDownMenu from "@radix-ui/react-dropdown-menu";
import { Check, X } from "lucide-react";
import { cn } from "../utils/cn";
import { useCallback, useState } from "react";

type KeyLike = string | number;

interface ChipSelectorProps<V, O = V> {
  values: V[];
  options: O[];
  /** Unique, stable key for each value */
  getValueKey: (opt: V) => KeyLike;
  /** Unique, stable key for each option */
  getOptionKey: (opt: O) => KeyLike;
  /** Render how each selected chip looks */
  renderChip: (val: V) => React.ReactNode;
  /** Render how each option in the dropdown looks */
  renderOption: (val: O) => React.ReactNode;
  /** Called when an option is picked */
  onToggle: (val: O) => void;
  onRemove: (val: V) => void;

  onOpenChange?: (open: boolean) => void;

  disabled?: (opt: O) => boolean;
  reason?: (opt: O) => React.ReactNode;

  placeholder?: React.ReactNode;
  className?: string;
  menuClassName?: string;
  itemClassName?: string;
}

export function ChipSelector<V, O = V>({
  values,
  options,
  getValueKey,
  getOptionKey,
  renderChip,
  renderOption,
  onToggle,
  onOpenChange,
  onRemove,
  disabled,
  reason,
  placeholder = (
    <span className="text-sm text-muted-foregorund">Select...</span>
  ),
  className,
  menuClassName,
  itemClassName,
}: ChipSelectorProps<V, O>) {
  const [open, setOpen] = useState(false);
  const handleOpenChange = useCallback(
    (next: boolean) => {
      setOpen(next);
      onOpenChange?.(next);
    },
    [onOpenChange],
  );
  const selectedKeys = new Set<KeyLike>(values.map(getValueKey));
  return (
    <DropDownMenu.Root
      modal={false}
      open={open}
      onOpenChange={handleOpenChange}
    >
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
          "z-50 mt-1 w-64 max-h-96 overflow-y-auto rounded border bg-popover p-1 shadow",
          menuClassName,
        )}
      >
        {options.map((option) => {
          const k = getOptionKey(option);
          const checked = selectedKeys.has(k);
          const isDisabled = disabled?.(option) ?? false;
          const disabledReason = isDisabled ? reason?.(option) : null;

          return (
            <DropDownMenu.CheckboxItem
              key={k}
              checked={checked}
              onCheckedChange={() => {
                if (!isDisabled) onToggle(option);
              }}
              className={cn(
                "relative flex cursor-pointer select-non items-center gap-2 rounded px-2 py-2 outline-none",
                isDisabled
                  ? "cursor-not-allowed opacity-80 text-muted-foreground"
                  : "cursor-pointer hover:bg-accent focus:bg-accent",
                itemClassName,
              )}
            >
              <span className="flex flex-col min-w-0 items-start gap-1">
                <span className="flex min-w-0 items-center gap-2">
                  {renderOption(option)}
                </span>
                {reason ? (
                  <span className="text-xs leading-snug text-red-500">
                    {disabledReason}
                  </span>
                ) : null}
              </span>

              {!isDisabled && (
                <DropDownMenu.ItemIndicator className="ml-auto">
                  <Check className="h-4 w-4" aria-hidden />
                </DropDownMenu.ItemIndicator>
              )}
            </DropDownMenu.CheckboxItem>
          );
        })}
      </DropDownMenu.Content>
    </DropDownMenu.Root>
  );
}
