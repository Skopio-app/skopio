const SkeletonChart = () => {
  return (
    <div className="h-[200px] w-full relative flex gap-1 items-end px-4 animate-pulse">
      {Array.from({ length: 12 }).map((_, idx) => (
        <div
          key={idx}
          className="flex-1 bg-gray-300/60 rounded-md"
          style={{
            height: `${40 + Math.random() * 100}px`,
            minWidth: "6px",
          }}
        />
      ))}
    </div>
  );
};

export default SkeletonChart;
