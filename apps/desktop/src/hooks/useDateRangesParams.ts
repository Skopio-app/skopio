import {
  DATE_RANGE_LABELS,
  DateRangeType,
  getRangeDates,
  mapRangeToPreset,
} from "@/utils/time";
import { addDays, startOfDay } from "date-fns";
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

  const isCustom = selectedRange === DateRangeType.Custom;

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

  const [startDate, endDate] = useMemo(
    () => getRangeDates(selectedRange, customStart, customEnd),
    [selectedRange, customStart, customEnd],
  );

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
