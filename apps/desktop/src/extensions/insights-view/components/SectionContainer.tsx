import SkeletonChart from "@/components/loading/SkeletonChart";

interface SectionContainerProps {
  title?: string;
  children: React.ReactNode;
  loading: boolean;
}

const SectionContainer: React.FC<SectionContainerProps> = ({
  title,
  children,
  loading,
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
        <SkeletonChart />
      ) : (
        <div className="flex-1 overflow-hidden">{children}</div>
      )}
    </div>
  );
};

export default SectionContainer;
