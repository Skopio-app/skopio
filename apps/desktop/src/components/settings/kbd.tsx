import { cn } from "@skopio/ui";

const Kbd: React.FC<React.HTMLAttributes<HTMLSpanElement>> = ({
  className,
  children,
  ...props
}) => (
  <kbd
    className={cn(
      "inline-flex items-center rounded-md border px-2 py-1 text-xs font-mono shadow-sm",
      "bg-muted text-muted-foreground",
      className,
    )}
    {...props}
  >
    {children}
  </kbd>
);

export default Kbd;
