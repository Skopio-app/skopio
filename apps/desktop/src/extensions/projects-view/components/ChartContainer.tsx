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
    <div className="relative flex flex-col rounded-2xl w-full max-w-3xl">
      {title && (
        <div className="flex items-center justify-center py-2">
          <h2 className="text-sm font-medium text-muted-foreground text-center">
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

export default ChartContainer;
