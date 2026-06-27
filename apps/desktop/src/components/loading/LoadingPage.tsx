import BorderBeam from "./BorderBeam";
import MatrixLoading from "./MatrixLoading";

const LoadingPage = () => {
  return (
    <div className="relative h-dvh w-full overflow-hidden bg-background">
      <MatrixLoading fit="cover" height="100%" width="100%" />
      <BorderBeam loading className="inset-1 z-50 rounded-lg" />
    </div>
  );
};

export default LoadingPage;
