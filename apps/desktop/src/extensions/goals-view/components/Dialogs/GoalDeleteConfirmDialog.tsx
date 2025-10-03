import * as Dialog from "@radix-ui/react-dialog";
import { Button } from "@skopio/ui";
import { X } from "lucide-react";
import { useGoalStore } from "../../stores/useGoalStore";

interface GoalDeleteConfirmDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  goalName: string;
  goalId: number;
}

const GoalDeleteConfirmDialog: React.FC<GoalDeleteConfirmDialogProps> = ({
  open,
  onOpenChange,
  goalName,
  goalId,
}) => {
  const { deleteGoal } = useGoalStore();

  const handleConfirm = async () => {
    await deleteGoal(goalId);
  };

  return (
    <Dialog.Root open={open} onOpenChange={onOpenChange}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-foreground/50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 max-w-md w-[90vw] -translate-x-1/2 -translate-y-1/2 bg-background rounded-md p-6 shadow-xl focus:outline-none z-60">
          <div className="flex justify-between items-center mb-4">
            <Dialog.Title className="text-lg font-medium text-foreground">
              Confirm Deletion
            </Dialog.Title>
            <Dialog.Description className="sr-only">
              Confirmation dialog for whether to proceed with goal deletion.
            </Dialog.Description>
            <Dialog.Close asChild>
              <Button
                variant="ghost"
                className="text-muted-foreground hover:text-foreground"
                aria-label="Close"
              >
                <X className="w-5 h-5" />
              </Button>
            </Dialog.Close>
          </div>

          <p className="text-sm text-foreground mb-6">
            Are you sure you want to delete the goal <strong>{goalName}</strong>
            ? This action cannot be undone.
          </p>

          <div className="flex justify-end gap-3">
            <Dialog.Close asChild>
              <Button variant="outline">Cancel</Button>
            </Dialog.Close>
            <Button variant="destructive" onClick={handleConfirm}>
              Delete
            </Button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default GoalDeleteConfirmDialog;
