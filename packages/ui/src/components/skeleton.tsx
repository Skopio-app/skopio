import { cn } from "../utils/cn";

function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      className={cn(
        "bg-gray-100 dark:bg-gray-200 animate-pulse rounded-md",
        "h-4 w-full",
        className,
      )}
      {...props}
    />
  );
}

export { Skeleton };
