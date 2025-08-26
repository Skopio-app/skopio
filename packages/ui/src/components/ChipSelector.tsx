import * as DropDownMenu from "@radix-ui/react-dropdown-menu";
import { X } from "lucide-react";

interface ChipSelectorProps<T> {
  values: T[];
  options: T[];
  onToggle: (val: T) => void;
  onRemove: (val: T) => void;
  getLabel: (val: T) => string;
}

export function ChipSelector<T>({
  values,
  options,
  onToggle,
  onRemove,
  getLabel,
}: ChipSelectorProps<T>) {
  return (
    <DropDownMenu.Root modal={false}>
      <DropDownMenu.Trigger asChild>
        <div className="flex flex-wrap items-center gap-1 px-3 py-1 border rounded cursor-pointer max-w-full">
          {values.map((item) => (
            <span
              key={getLabel(item)}
              className="flex items-center gap-1 px-2 py-1 bg-gray-200 rounded whitespace-nowrap"
              onClick={(e) => {
                e.stopPropagation();
                onRemove(item);
              }}
            >
              {getLabel(item)} <X className="w-3 h-3" />
            </span>
          ))}
        </div>
      </DropDownMenu.Trigger>
      <DropDownMenu.Content className="z-50 mt-1 w-48 max-h-60 overflow-y-auto bg-white border rounded shadow">
        {options.map((option) => (
          <DropDownMenu.Item
            key={getLabel(option)}
            onSelect={() => onToggle(option)}
            className="px-3 py-2 cursor-pointer hover:bg-gray-100"
          >
            {getLabel(option)}
          </DropDownMenu.Item>
        ))}
      </DropDownMenu.Content>
    </DropDownMenu.Root>
  );
}
