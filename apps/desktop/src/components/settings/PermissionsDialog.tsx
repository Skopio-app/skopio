import * as Dialog from "@radix-ui/react-dialog";
import { usePermissions } from "@/hooks/usePermissions";
import { Button } from "@skopio/ui";
import { PermissionRow } from "./PermissionRow";
import { useState } from "react";
import { isGranted } from "@/utils/data";

const LOCALSTORAGE_KEY = "permission-dialog-seen";

const readSeen = () => {
  if (typeof window === "undefined") return false;
  return !!localStorage.getItem(LOCALSTORAGE_KEY);
};

const PermissionsDialog = () => {
  const { summary, loading } = usePermissions();
  const [seen, setSeen] = useState(readSeen);

  const needsAccessibility = !!summary && !isGranted(summary.accessibility);
  const needsInput = !!summary && !isGranted(summary.inputMonitoring);
  const needsAny = needsAccessibility || needsInput;

  const shouldOpen = (needsAccessibility && needsInput) || !seen;

  if (loading || !summary || !needsAny) return null;

  const handleOpenChange = (nextOpen: boolean) => {
    if (!nextOpen) {
      localStorage.setItem(LOCALSTORAGE_KEY, "1");
      setSeen(true);
    }
  };

  return (
    <Dialog.Root defaultOpen open={shouldOpen} onOpenChange={handleOpenChange}>
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
