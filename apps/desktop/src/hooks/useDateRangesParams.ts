import {
  DATE_RANGE_LABELS,
  DateRangeType,
  getRangeDates,
  mapRangeToPreset,
} from "@/utils/time";
import { addDays, differenceInMilliseconds, startOfDay } from "date-fns";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useSearchParams } from "react-router";

export type UseDateRangeParams = {
  selectedRange: DateRangeType;
  setSelectedRange: (r: DateRangeType) => void;

  startDate: Date;
  endDate: Date;

  isCustom: boolean;
  customStart: Date;
  customEnd: Date;
  pendingStart: Date;
  pendingEnd: Date;
  setPendingStart: (d: Date) => void;
  setPendingEnd: (d: Date) => void;
  setCustomStart: (d: Date) => void;
  setCustomEnd: (d: Date) => void;

  applyPresetToStore: (storeSetter: (preset: any) => void) => void;
};

export const useDateRangeParams = (): UseDateRangeParams => {
  const [searchParams, setSearchParams] = useSearchParams();

  // init from URL
  const paramRange = searchParams.get("range") as DateRangeType;
  const initialRange: DateRangeType =
    paramRange && DATE_RANGE_LABELS.includes(paramRange)
      ? paramRange
      : DateRangeType.Today;

  const [selectedRange, setSelectedRange] =
    useState<DateRangeType>(initialRange);

  // Custom range state
  const [customStart, _setCustomStart] = useState<Date>(new Date());
  const [customEnd, _setCustomEnd] = useState<Date>(new Date());
  const [pendingStart, setPendingStart] = useState<Date>(customStart);
  const [pendingEnd, setPendingEnd] = useState<Date>(customEnd);
  const [relativeRangeClock, setRelativeRangeClock] = useState<number>(() =>
    Date.now(),
  );

  const isCustom = selectedRange === DateRangeType.Custom;

  useEffect(() => {
    if (isCustom) return;

    const refreshRelativeRange = () => setRelativeRangeClock(Date.now());
    const refreshOnVisibility = () => {
      if (!document.hidden) refreshRelativeRange();
    };

    const now = new Date();
    const nextDay = addDays(startOfDay(now), 1);
    const timeoutMs = Math.max(
      1_000,
      differenceInMilliseconds(nextDay, now) + 1_000,
    );

    const timeoutId = window.setTimeout(refreshRelativeRange, timeoutMs);

    window.addEventListener("focus", refreshRelativeRange);
    document.addEventListener("visibilitychange", refreshOnVisibility);

    return () => {
      window.clearTimeout(timeoutId);
      window.removeEventListener("focus", refreshRelativeRange);
      document.removeEventListener("visibilitychange", refreshOnVisibility);
    };
  }, [isCustom, relativeRangeClock]);

  useEffect(() => {
    setSearchParams(
      (prev) => {
        const next = new URLSearchParams(prev);
        if (next.get("range") === selectedRange) return prev;
        next.set("range", selectedRange);
        return next;
      },
      { replace: true },
    );
  }, [selectedRange, setSearchParams]);

  const setCustomStart = useCallback(
    (d: Date) => {
      _setCustomStart(d);
      if (isCustom) {
        const maxEnd = addDays(startOfDay(d), 30);
        _setCustomEnd((prev) => (prev > maxEnd ? maxEnd : prev));
      }
    },
    [isCustom],
  );

  const setCustomEnd = useCallback(
    (d: Date) => {
      if (!isCustom) {
        _setCustomEnd(d);
        return;
      }
      const maxEnd = addDays(startOfDay(customStart), 30);
      _setCustomEnd(d > maxEnd ? maxEnd : d);
    },
    [isCustom, customStart],
  );

  const [startDate, endDate] = useMemo(() => {
    void relativeRangeClock;
    return getRangeDates(selectedRange, customStart, customEnd);
  }, [selectedRange, customStart, customEnd, relativeRangeClock]);

  // Helper to push preset to a store without re-implementing this in every screen
  const applyPresetToStore = (storeSetter: (preset: any) => void) => {
    const preset = mapRangeToPreset(selectedRange, startDate, endDate);
    storeSetter(preset);
  };

  return {
    selectedRange,
    setSelectedRange,
    startDate,
    endDate,
    isCustom,
    customStart,
    customEnd,
    pendingStart,
    pendingEnd,
    setPendingStart,
    setPendingEnd,
    setCustomStart,
    setCustomEnd,
    applyPresetToStore,
  };
};
