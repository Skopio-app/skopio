import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@skopio/ui";
import AverageDaySection from "./components/AverageDaySection";
import ActivitySection from "./components/ActivitySection";
import DailyAverageSection from "./components/DailyAverageSection";
import { useEffect, useState } from "react";
import { commands } from "../../types/tauri.gen";
import { toast } from "sonner";

const InsightsView = () => {
  const [years, setYears] = useState<number[]>([]);
  // const [selectedYear, setSelectedYear] = useState<string | null>(null);

  useEffect(() => {
    commands.fetchActiveYears().then(setYears).catch(toast.error);
  }, []);

  return (
    <div className="flex flex-col m-3 space-y-5">
      <h2 className="text-neutral-800 font-semibold mb-4 mt-3 text-lg">
        Insights
      </h2>
      <Select>
        <SelectTrigger className="w-[180px]">
          <SelectValue placeholder="Select a year" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectLabel>Year</SelectLabel>
            {years.map((year) => (
              <SelectItem key={year} value={String(year)}>
                {year}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>

      <AverageDaySection />
      <ActivitySection />
      <DailyAverageSection />
    </div>
  );
};

export default InsightsView;
