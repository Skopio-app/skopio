import * as Dialog from "@radix-ui/react-dialog";
import { usePermissions } from "../../hooks/usePermissions";
import { PermissionCard } from "./PermissionCard";
import { Keyboard, PersonStanding } from "lucide-react";
import { Button } from "@skopio/ui";

const PermissionsDialog = () => {
  const {
    summary,
    loading,
    busy,
    requestAccessibility,
    requestInputMonitoring,
  } = usePermissions();

  if (loading || !summary) return null;

  const needsAccessibility = summary.accessibility !== "Granted";
  const needsInput = summary.inputMonitoring !== "Granted";
  const needsAny = needsAccessibility || needsInput;

  if (!needsAny) return null;

  return (
    <Dialog.Root defaultOpen>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40" />
        <Dialog.Content className="fixed left-1/2 top-1/2 w-[680px] max-w-[90vw] -translate-x-1/2 -translate-y-1/2 rounded-xl bg-white p-5 shadow-xl">
          <div className="mb-4">
            <h2 className="text-lg font-semibold">Permissions required</h2>
            <p className="text-sm text-muted-foreground">
              Skopio needs the following permissions to track activity
              accurately.
            </p>
          </div>

          <div className="space-y-3">
            {needsAccessibility && (
              <PermissionCard
                icon={<PersonStanding className="h-5 w-5 text-blue-400" />}
                title="Accessibility"
                description={
                  <>
                    Required for detecting the frontmost app and reading its
                    window title. If you previously clicked “Don’t Allow”,
                    you’ll need to grant it from System Settings.
                  </>
                }
                status={
                  summary.accessibility === "Granted" ? "granted" : "denied"
                }
                loading={busy === "accessibility"}
                onGrant={async () => {
                  await requestAccessibility();
                }}
              />
            )}

            {needsInput && (
              <PermissionCard
                icon={<Keyboard className="h-5 w-5 text-neutral-600" />}
                title="Input Monitoring"
                description="Required to detect keyboard and mouse activity during AFK checks."
                status={
                  summary.inputMonitoring === "Granted" ? "granted" : "denied"
                }
                loading={busy === "inputMonitoring"}
                onGrant={async () => {
                  await requestInputMonitoring();
                }}
              />
            )}
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
