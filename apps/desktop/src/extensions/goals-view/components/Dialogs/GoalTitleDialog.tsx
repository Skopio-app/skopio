import * as Dialog from "@radix-ui/react-dialog";
import { Button, Input } from "@skopio/ui";
import { X } from "lucide-react";
import z from "zod/v4";
import { GoalUpdateInput } from "@/types/tauri.gen";
import { useGoalStore } from "../../stores/useGoalStore";
import { FieldErrors, useForm } from "react-hook-form";
import { standardSchemaResolver } from "@hookform/resolvers/standard-schema";
import { toast } from "sonner";

interface GoalTitleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  goalId: number;
}

const titleFormSchema = z.object({
  name: z
    .string()
    .trim()
    .min(1, { message: "Title is required" })
    .max(200, { message: "Title must be under 200 characters" }),
});

const GoalTitleDialog: React.FC<GoalTitleDialogProps> = ({
  open,
  onOpenChange,
  goalId,
  title,
}) => {
  const { updateGoal } = useGoalStore();

  const form = useForm<z.infer<typeof titleFormSchema>>({
    resolver: standardSchemaResolver(titleFormSchema),
    defaultValues: {
      name: title,
    },
  });

  const { register, handleSubmit, setValue } = form;

  const onSubmit = async (data: z.infer<typeof titleFormSchema>) => {
    const input: GoalUpdateInput = {
      name: data.name,
    };

    const success = await updateGoal(goalId, input);
    if (success) onOpenChange(false);
  };

  const onInvalid = (errors: FieldErrors<typeof titleFormSchema>) => {
    const firstError = Object.values(errors)[0];
    if (firstError && "message" in firstError) {
      toast.error(firstError.message as string);
    } else {
      toast.error("Please fix the highlighted errors.");
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-foreground/50" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-w-xl w-[90vw] max-h-[90vh] h-[200px] overflow-y-auto -translate-x-1/2 -translate-y-1/2 bg-background p-4 rounded-md shadow-lg focus:outline-none z-60 bg-sidebar border border-muted">
          <div className="flex justify-between items-start mb-2">
            <Dialog.Title className="text-xl font-semibold text-foreground">
              Edit Goal Title
            </Dialog.Title>
            <Dialog.Description className="sr-only">
              Edit goal name
            </Dialog.Description>
            <Dialog.Close asChild>
              <Button variant="ghost" aria-label="Close">
                <X className="w-5 h-5" />
              </Button>
            </Dialog.Close>
          </div>
          <form onSubmit={handleSubmit(onSubmit, onInvalid)}>
            <div className="mb-4">
              <Input
                type="text"
                {...register("name", {
                  onChange: (e) => {
                    setValue("name", e.target.value, {
                      shouldValidate: true,
                      shouldDirty: true,
                    });
                  },
                })}
                className="mt-1 block w-full rounded-md border border-border px-3 py-2 shadow-sm focus:border-primary focus:ring-primary text-sm"
                autoFocus
                required
              />
            </div>

            <div className="flex justify-end gap-2">
              <Dialog.Close asChild>
                <Button type="button" variant="ghost">
                  Cancel
                </Button>
              </Dialog.Close>
              <Button type="submit">OK</Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default GoalTitleDialog;
