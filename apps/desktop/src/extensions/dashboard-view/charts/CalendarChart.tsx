import { CalendarTooltipProps, ResponsiveCalendar } from "@nivo/calendar";
import { format, startOfYear } from "date-fns";
import { formatDuration } from "../../../utils/time";

interface CalendarChartProps {
  data: {
    value: number;
    day: string;
  }[];
}

const CalendarTooltip: React.FC<CalendarTooltipProps> = ({ day, value }) => {
  const date = new Date(day);

  return (
    <div className="rounded-md bg-white p-2 text-xs shadow-md w-32">
      <div className="font-medium">{format(date, "EEEE, MMM d")}</div>
      <div className="text-gray-500">{formatDuration(parseInt(value))}</div>
    </div>
  );
};

const CalendarChart: React.FC<CalendarChartProps> = ({ data }) => {
  return (
    <div className="h-[200px] w-full">
      <ResponsiveCalendar
        data={data}
        from={startOfYear(new Date())}
        to={new Date()}
        emptyColor="#eeeeee"
        colors={["#4ade80"]}
        margin={{ top: 40, right: 40, bottom: 40, left: 40 }}
        monthBorderColor="#ffffff"
        dayBorderWidth={2}
        dayBorderColor="#ffffff"
        tooltip={CalendarTooltip}
        legends={[
          {
            anchor: "bottom-left",
            direction: "row",
            translateY: 36,
            itemCount: 4,
            itemWidth: 42,
            itemHeight: 36,
            itemsSpacing: 14,
            itemDirection: "right-to-left",
            //TODO: Add full legend once goal logic is configured
            // data: [
            //     {
            //         id: "test",
            //         label: "Active time",
            //         color: "#4ade80",
            //     },
            // ],
          },
        ]}
      />
    </div>
  );
};

export default CalendarChart;
