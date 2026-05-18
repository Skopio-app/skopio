import { createContext, useContext } from "react";
import type { TourId } from "./TourSteps";

type TourContextValue = {
  startTour: (tour?: TourId) => void;
};

export const TourContext = createContext<TourContextValue | null>(null);

export const useGuidedTour = () => {
  const value = useContext(TourContext);
  if (!value) throw new Error("useGuidedTour must be used inside TourProvider");
  return value;
};
