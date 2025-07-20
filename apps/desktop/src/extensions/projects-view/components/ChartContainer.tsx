import SkeletonChart from "../../../components/SkeletonChart";

interface ChartContainerProps {
  title?: string;
  children: React.ReactNode;
  loading: boolean;
}

const ChartContainer: React.FC<ChartContainerProps> = ({
  title,
  children,
  loading,
}) => {
  return (
    <div className="relative flex rounded-2xl border border-muted w-full max-w-3xl">
      <div className="flex items-center justify-between mb-2">
        {title && (
          <h2 className="text-sm font-medium text-muted-foreground">{title}</h2>
        )}
      </div>

      {loading ? (
        <SkeletonChart />
      ) : (
        <div className="flex-1 overflow-hidden">{children}</div>
      )}
    </div>
  );
};

export default ChartContainer;
