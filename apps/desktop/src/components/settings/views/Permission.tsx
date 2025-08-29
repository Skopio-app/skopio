import { Keyboard, PersonStanding } from "lucide-react";
import { PermissionCard } from "../PermissionCard";
import { usePermissions } from "../../../hooks/usePermissions";

const Permission = () => {
  const {
    summary,
    busy,
    requestAccessibility,
    requestInputMonitoring,
    openSettings,
  } = usePermissions();
  return (
    <div className="space-y-3">
      <PermissionCard
        icon={<PersonStanding className="h-5 w-5 text-blue-400" />}
        title="Accessibility"
        description="Skopio needs Accessibility permission to detect the active window."
        loading={busy === "accessibility"}
        status={summary?.accessibility === "Granted" ? "granted" : "denied"}
        onGrant={async () => {
          const result = await requestAccessibility();
          if (result !== "Granted") {
            await openSettings("accessibility");
          }
        }}
      />
      <PermissionCard
        icon={<Keyboard className="h-5 w-5 text-neutral-600" />}
        title="Input Monitoring"
        description="Allows Skopio to detect keyboard/mouse activity to measure idle times."
        status={summary?.inputMonitoring === "Granted" ? "granted" : "denied"}
        onGrant={async () => {
          const result = await requestInputMonitoring();
          if (result !== "Granted") {
            await openSettings("inputMonitoring");
          }
        }}
      />
    </div>
  );
};

export default Permission;
