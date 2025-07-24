import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Controller, useForm } from "react-hook-form";
import { Button, Checkbox, Label, ScrollArea, Separator } from "@skopio/ui";
import { toast } from "sonner";

interface BranchSelectionDialogProps {
  branches: string[];
  selectedBranch: string[] | null;
  onSelect: (branch: string[] | null) => void;
}

interface FormValues {
  selectedBranches: Record<string, boolean>;
  selectAll: boolean;
}

const BranchSelectionDialog: React.FC<BranchSelectionDialogProps> = ({
  branches,
  selectedBranch,
  onSelect,
}) => {
  const [open, setOpen] = useState(false);

  const defaultValues: FormValues = {
    selectAll: !selectedBranch,
    selectedBranches: Object.fromEntries(
      branches.map((b) => [b, selectedBranch?.includes(b) ?? true]),
    ),
  };

  const { control, watch, setValue, handleSubmit } = useForm<FormValues>({
    defaultValues,
  });

  const selectAll = watch("selectAll");
  const selectedBranches = watch("selectedBranches");

  const canSubmit =
    selectAll || Object.values(selectedBranches ?? {}).some((v) => v === true);

  const onSubmit = (data: FormValues) => {
    if (data.selectAll) {
      onSelect(null);
      setOpen(false);
    } else {
      const selected = Object.entries(data.selectedBranches)
        .filter(([, value]) => value)
        .map(([key]) => key);

      if (selected.length === 0) {
        toast.error("Select at least one branch");
        return;
      }

      onSelect(selected);
      setOpen(false);
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger asChild>
        <button className="underline decoration-dotted underline-offset-4 text-blue-600 hover:text-blue-800">
          {selectedBranch ?? "all"}
        </button>
      </Dialog.Trigger>

      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50">
          <Dialog.Content className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-white p-6 rounded-lg shadow-md w-56">
            <Dialog.Title>Select branches</Dialog.Title>
            <Dialog.Description className="sr-only">
              Choose which branch to display activity for.
            </Dialog.Description>

            <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
              <Controller
                control={control}
                name="selectAll"
                render={({ field }) => (
                  <div className="flex items-center space-x-2">
                    <Checkbox
                      id="select-all"
                      checked={field.value}
                      onCheckedChange={(value) => {
                        field.onChange(!!value);
                        branches.forEach((b) => {
                          setValue(`selectedBranches.${b}`, !!value);
                        });
                      }}
                    />
                    <Label htmlFor="select-all" className="text-sm font-medium">
                      Select All
                    </Label>
                  </div>
                )}
              />

              <Separator className="my-2 bg-neutral-900" />

              <ScrollArea className="max-h-64 pr-2">
                <div className="space-y-2">
                  {branches.map((branch) => (
                    <Controller
                      key={branch}
                      control={control}
                      name={`selectedBranches.${branch}` as const}
                      render={({ field }) => (
                        <div className="flex items-center space-x-2">
                          <Checkbox
                            id={`branch-${branch}`}
                            checked={field.value}
                            onCheckedChange={(value) => field.onChange(!!value)}
                          />
                          <Label
                            htmlFor={`branch-${branch}`}
                            className="text-sm"
                          >
                            {branch}
                          </Label>
                        </div>
                      )}
                    />
                  ))}
                </div>
              </ScrollArea>

              <div className="flex justify-end gap-2 pt-4">
                <Dialog.Close asChild>
                  <Button type="button" variant="outline">
                    Cancel
                  </Button>
                </Dialog.Close>

                <Button type="submit" disabled={!canSubmit}>
                  Apply
                </Button>
              </div>
            </form>
          </Dialog.Content>
        </Dialog.Overlay>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default BranchSelectionDialog;
