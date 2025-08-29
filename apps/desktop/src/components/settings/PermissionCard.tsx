import { Button, cn } from "@skopio/ui";

type PermissionState = "granted" | "denied" | "unknown";

export interface PermissionCardProps {
  icon: React.ReactNode;
  title: string;
  description?: React.ReactNode;
  status: PermissionState;
  onGrant: () => Promise<void> | void;
  loading?: boolean;
  error?: string | null;
  actionsRight?: React.ReactNode;
  grantLabel?: string;
  className?: string;
}

const badgeClass = (status: PermissionState) => {
  switch (status) {
    case "granted":
      return "bg-green-600/10 text-green-700 dark:text-green-400 border border-green-600/20";
    case "denied":
      return "bg-red-600/10 text-red-700 dark:text-400 border-red-600/20";
    case "unknown":
      return "bg-amber-500/10 text-amber-700 dark:text-amber-300 border border-amber-500/20";
  }
};

export const PermissionCard: React.FC<PermissionCardProps> = ({
  icon,
  title,
  description,
  status,
  onGrant,
  loading = false,
  error = null,
  className,
}) => {
  const showGrant = status !== "granted";

  return (
    <div
      className={cn(
        "flex w-full items-center gap-4 rounded-xl border border-neutral-400/60 bg-muted/70 p-4 shadow-sm transition-shadow",
        className,
      )}
    >
      <div className="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-muted">
        {icon}
      </div>

      <div className="flex flex-1 items-center justify-between gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h3 className="text-sm font-semibold leading-tight">{title}</h3>
            <span
              className={cn(
                "rounded-full px-2 py-0.5 text-xs",
                badgeClass(status),
              )}
            >
              {status === "granted"
                ? "Granted"
                : status === "denied"
                  ? "Denied"
                  : "Not determined"}
            </span>
          </div>

          {description ? (
            <p className="mt-1 text-xs text-muted-foreground">{description}</p>
          ) : null}

          {error ? (
            <p className="mt-2 text-xs text-red-600 dark:text-red-400">
              {error}
            </p>
          ) : null}
        </div>

        <div className="items-center justify-center gap-2">
          {/* {actionsRight} */}
          {showGrant ? (
            <Button variant="outline" onClick={onGrant} disabled={loading}>
              Grant Access
            </Button>
          ) : null}
        </div>
      </div>
    </div>
  );
};
