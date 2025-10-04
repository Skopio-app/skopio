import * as Dialog from "@radix-ui/react-dialog";
import { usePermissions } from "@/hooks/usePermissions";
import { Button } from "@skopio/ui";
import { PermissionRow } from "./PermissionRow";
import { useEffect, useState } from "react";
import { isGranted } from "@/utils/data";

const LOCALSTORAGE_KEY = "permission-dialog-seen";

const PermissionsDialog = () => {
  const { summary, loading } = usePermissions();
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (loading || !summary) return;

    const needsAccessibility = !isGranted(summary.accessibility);
    const needsInput = !isGranted(summary.inputMonitoring);
    const needsBoth = needsAccessibility && needsInput;

    if (needsBoth || !localStorage.getItem(LOCALSTORAGE_KEY)) {
      setOpen(true);
    }
  }, [loading, summary]);

  const handleClose = () => {
    localStorage.setItem(LOCALSTORAGE_KEY, "1");
    setOpen(false);
  };

  if (loading || !summary) return null;

  const needsAccessibility = !isGranted(summary.accessibility);
  const needsInput = !isGranted(summary.inputMonitoring);
  const needsAny = needsAccessibility || needsInput;

  if (!needsAny) return null;

  return (
    <Dialog.Root defaultOpen open={open} onOpenChange={handleClose}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-foreground/40" />
        <Dialog.Content
          className="fixed left-1/2 top-1/2 w-[680px] max-w-[90vw] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-sidebar p-5 shadow-xl border border-muted"
          onEscapeKeyDown={(e) => e.preventDefault()}
          onInteractOutside={(e) => e.preventDefault()}
        >
          <div className="mb-4">
            <Dialog.Title className="text-lg font-semibold text-foreground">
              Permissions required
            </Dialog.Title>
            <Dialog.Description className="text-sm text-muted-foreground">
              Skopio needs the following permissions to track app activity
              accurately.
            </Dialog.Description>
          </div>

          <div className="space-y-3">
            {needsAccessibility && <PermissionRow kind="accessibility" />}
            {needsInput && <PermissionRow kind="inputMonitoring" />}
          </div>

          <div className="mt-5 flex justify-end">
            <Dialog.Close asChild>
              <Button variant="secondary">Done</Button>
            </Dialog.Close>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};

export default PermissionsDialog;
