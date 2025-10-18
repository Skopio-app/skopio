const SkeletonChart = () => {
  const barHeight = (i: number) => {
    const x = Math.sin(i * 9377) * 10000;
    const frac = x - Math.floor(x);
    return 40 + Math.round(frac * 100);
  };

  return (
    <div className="h-[200px] w-full relative flex gap-1 items-end px-4 animate-pulse">
      {Array.from({ length: 12 }).map((_, idx) => (
        <div
          key={idx}
          className="flex-1 bg-gray-300/60 rounded-md"
          style={{
            height: `${barHeight(idx)}px`,
            minWidth: "6px",
          }}
        />
      ))}
    </div>
  );
};

export default SkeletonChart;
