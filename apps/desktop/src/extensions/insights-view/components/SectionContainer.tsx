import SkeletonChart, {
  SkeletonChartVariant,
} from "@/components/loading/SkeletonChart";
import { Skeleton } from "@skopio/ui";

type SectionVariant = "chart" | "text";

interface SectionContainerProps {
  title?: string;
  children: React.ReactNode;
  loading: boolean;
  variant?: SectionVariant;
  skeletonVariant?: SkeletonChartVariant;
}

const SectionContainer: React.FC<SectionContainerProps> = ({
  title,
  children,
  loading,
  variant = "chart",
  skeletonVariant = "bar",
}) => {
  return (
    <div className="relative flex flex-col rounded-2xl w-full border border-[var(--muted-foreground)]">
      {title && (
        <div className="flex items-center justify-start py-2">
          <h2 className="text-lg font-light text-muted-foreground text-center ml-3">
            {title}
          </h2>
        </div>
      )}

      {loading ? (
        variant === "chart" ? (
          <SkeletonChart variant={skeletonVariant} />
        ) : (
          <div className="flex flex-col gap-2 p-3">
            <Skeleton className="h-4 w-40" />
            <Skeleton className="h-4 w-2/5" />
          </div>
        )
      ) : (
        <div className="flex-1 overflow-hidden">{children}</div>
      )}
    </div>
  );
};

export default SectionContainer;
