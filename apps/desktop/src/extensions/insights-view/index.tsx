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

const InsightsView = () => {
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
            <SelectItem value="2023">2023</SelectItem>
            <SelectItem value="2024">2024</SelectItem>
            <SelectItem value="2025">2025</SelectItem>
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
