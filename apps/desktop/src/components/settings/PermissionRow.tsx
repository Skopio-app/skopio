import { Keyboard, PersonStanding } from "lucide-react";
import type { ReactNode } from "react";
import { usePermissions } from "@/hooks/usePermissions";
import { PermissionCard } from "@/components/settings/PermissionCard";
import { PermissionKind } from "@/types/tauri.gen";
import { isGranted } from "@/utils/data";

type PermissionMeta = {
  title: string;
  icon: ReactNode;
  description: string;
  settingsKey?: PermissionKind;
};

const PERMISSIONS: Record<PermissionKind, PermissionMeta> = {
  accessibility: {
    title: "Accessibility",
    icon: <PersonStanding className="h-5 w-5 text-primary" />,
    description:
      "Skopio needs the accessibility permission to detect the frontmost app window.",
    settingsKey: "accessibility",
  },
  inputMonitoring: {
    title: "Input Monitoring",
    icon: <Keyboard className="h-5 w-5 text-muted-foreground" />,
    description:
      "Allows Skopio to detect keyboard/mouse activity to measure idle (AFK) durations.",
    settingsKey: "inputMonitoring",
  },
};

type PermissionRowProps = { kind: PermissionKind };

export const PermissionRow: React.FC<PermissionRowProps> = ({ kind }) => {
  const {
    summary,
    busy,
    requestAccessibility,
    requestInputMonitoring,
    openSettings,
  } = usePermissions();

  if (!summary) return null;

  const meta = PERMISSIONS[kind];
  const statusString = summary[kind];
  const status = isGranted(statusString) ? "granted" : "denied";

  const onGrant = async () => {
    if (kind === "accessibility") {
      await requestAccessibility();
    } else if (kind === "inputMonitoring") {
      const res = await requestInputMonitoring();
      if (res !== "granted" && meta.settingsKey) {
        await openSettings(meta.settingsKey);
      }
    }
  };

  return (
    <PermissionCard
      icon={meta.icon}
      title={meta.title}
      description={meta.description}
      status={status}
      loading={busy === kind}
      onGrant={onGrant}
    />
  );
};
