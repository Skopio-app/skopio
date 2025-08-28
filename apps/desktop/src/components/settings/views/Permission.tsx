import { Keyboard, PersonStanding } from "lucide-react";
import { PermissionCard } from "../PermissionCard";

const Permission = () => {
  return (
    <div className="space-y-3">
      <PermissionCard
        icon={<PersonStanding className="h-5 w-5 text-blue-400" />}
        title="Accessibility"
        description="Skopio needs Accessibility permission to detect the active window"
        status="unknown"
        onGrant={() => console.log("Grant")}
      />
      <PermissionCard
        icon={<Keyboard className="h-5 w-5 text-neutral-600" />}
        title="Input Monitoring"
        description="Allows Skopio to detect keyboard/mouse activity to measure idle times."
        status="granted"
        onGrant={() => console.log("Grant")}
      />
    </div>
  );
};

export default Permission;
