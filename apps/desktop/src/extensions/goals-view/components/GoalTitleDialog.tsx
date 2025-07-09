import * as Dialog from "@radix-ui/react-dialog";
import { Button, Input } from "@skopio/ui";
import { X } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { z } from "zod";
import { GoalUpdateInput } from "../../../types/tauri.gen";
import { useGoalStore } from "../stores/useGoalStore";

interface GoalTitleDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  goalId: number;
}

const titleSchema = z
  .string()
  .trim()
  .min(1, { message: "Title is required" })
  .max(200, { message: "Title must be under 100 characters" });

const GoalTitleDialog: React.FC<GoalTitleDialogProps> = ({
  open,
  onOpenChange,
  goalId,
  title,
}) => {
  const [value, setValue] = useState<string>(title);
  const { updateGoal } = useGoalStore();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const result = titleSchema.safeParse(value);
    if (!result.success) {
      toast.error(result.error.errors[0].message);
      return;
    } else {
      const input: GoalUpdateInput = {
        name: result.data,
      };

      await updateGoal(goalId, input);
    }

    onOpenChange(false);
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/50" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-w-xl w-[90vw] max-h-[90vh] h-[200px] overflow-y-auto -translate-x-1/2 -translate-y-1/2 bg-white p-4 rounded-md shadow-lg focus:outline-none">
          <div className="flex justify-between items-start mb-2">
            <Dialog.Title className="text-xl font-semibold">
              Edit Goal Title
            </Dialog.Title>
            <Dialog.Close asChild>
              <Button
                variant="ghost"
                className="text-gray-500 hover:text-black"
                aria-label="Close"
              >
                <X className="w-5 h-5" />
              </Button>
            </Dialog.Close>
          </div>
          <form onSubmit={handleSubmit}>
            <div className="mb-4">
              <Input
                value={value}
                onChange={(e) => setValue(e.target.value)}
                className="mt-1 block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-primary focus:ring-primary text-sm"
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
