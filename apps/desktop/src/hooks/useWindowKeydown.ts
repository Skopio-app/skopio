import { useEffect, useRef } from "react";

type UseWindowKeydownOptions = AddEventListenerOptions;

export const useWindowKeydown = (
  handler: (event: KeyboardEvent) => void,
  options?: UseWindowKeydownOptions,
) => {
  const handlerRef = useRef(handler);

  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    const listener = (event: KeyboardEvent) => {
      handlerRef.current(event);
    };

    window.addEventListener("keydown", listener, options);
    return () => window.removeEventListener("keydown", listener, options);
  }, [options]);
};
