import BorderBeam from "./BorderBeam";
import MatrixLoading from "./MatrixLoading";

const LoadingPage = () => {
  return (
    <div className="relative flex flex-col items-center justify-center h-dvh w-full overflow-hidden bg-transparent">
      <BorderBeam loading />
      <div className="relative w-full px-4 py-6 sm:px-10 sm:py-8">
        <div className="relative z-10 flex w-full flex-col items-center gap-6">
          <MatrixLoading />
          <div className="flex flex-col items-center">
            <p className="text-lg font-medium text-foreground animate-pulse">
              Starting the local server…
            </p>
            <p className="text-sm text-muted-foreground">
              This might take a few seconds.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default LoadingPage;
