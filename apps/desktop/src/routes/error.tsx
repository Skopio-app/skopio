import { Component, ReactNode } from "react";
import { isRouteErrorResponse, useRouteError } from "react-router";

export const ErrorPage = () => {
  const error = useRouteError();
  const message = isRouteErrorResponse(error)
    ? `${error.status} ${error.statusText}`
    : (error as any)?.message || "Something went wrong.";

  return (
    <div className="flex h-screen items-center justify-center p-6 text-center">
      <div>
        <h1 className="text-xl font-semibold text-red-600">Oops!</h1>
        <p className="mt-2 text-neutral-600">{message}</p>
      </div>
    </div>
  );
};

export class ErrorBoundary extends Component<
  { children: ReactNode },
  { hasError: boolean }
> {
  state = { hasError: false };
  static getDerivedStateFromError() {
    return { hasError: true };
  }
  render() {
    if (this.state.hasError) {
      return (
        <div className="flex h-screen items-center justify-center p-6 text-center">
          <div>
            <h1 className="text-xl font-semibold text-red-600">
              Something broke.
            </h1>
            <p className="mt-2 text-neutral-600">Try reloading the app</p>
          </div>
        </div>
      );
    }
    return this.props.children;
  }
}
