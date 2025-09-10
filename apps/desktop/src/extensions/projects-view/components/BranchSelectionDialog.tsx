import { useEffect, useMemo, useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { Controller, useForm } from "react-hook-form";
import { Button, Checkbox, Label, ScrollArea, Separator } from "@skopio/ui";
import { toast } from "sonner";
import { usePresetFilter } from "../stores/usePresetFilter";

interface FormValues {
  selectedBranches: Record<string, boolean>;
  selectAll: boolean;
}

const BranchSelectionDialog = () => {
  const [open, setOpen] = useState(false);
  const { branches, selectedBranches } = usePresetFilter();

  const defaultValues: FormValues = useMemo(
    () => ({
      selectAll: !selectedBranches,
      selectedBranches: Object.fromEntries(
        branches.map((b) => [b, selectedBranches?.includes(b) ?? true]),
      ),
    }),
    [branches, selectedBranches],
  );

  const { control, watch, setValue, handleSubmit, reset } = useForm<FormValues>(
    {
      defaultValues,
    },
  );

  useEffect(() => {
    reset(defaultValues);
  }, [defaultValues, reset]);

  const selectedMap = watch("selectedBranches");

  const allSelected =
    branches.length > 0 && branches.every((b) => !!selectedMap?.[b]);

  const canSubmit =
    allSelected || Object.values(selectedMap ?? {}).some((v) => v === true);

  const onSubmit = (data: FormValues) => {
    if (allSelected) {
      usePresetFilter.setState({ selectedBranches: null });
      setOpen(false);
      return;
    }
    const selected = Object.entries(data.selectedBranches)
      .filter(([, value]) => value)
      .map(([key]) => key);

    if (selected.length === 0) {
      toast.error("Select at least one branch");
      return;
    }
    usePresetFilter.setState({ selectedBranches: selected });
    setOpen(false);
  };

  return (
    <Dialog.Root open={open} onOpenChange={setOpen}>
      <Dialog.Trigger asChild>
        <button className="underline decoration-dotted underline-offset-4 text-blue-600 hover:text-blue-800">
          {selectedBranches?.join(", ") ?? "all"}
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
                      checked={allSelected}
                      onCheckedChange={(value) => {
                        field.onChange(!!value);
                        branches.forEach((b) => {
                          setValue(`selectedBranches.${b}`, !!value, {
                            shouldDirty: true,
                            shouldTouch: true,
                          });
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
