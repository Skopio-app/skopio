import { CalendarTooltipProps, ResponsiveCalendar } from "@nivo/calendar";
import { format } from "date-fns";
import { formatDuration } from "@/utils/time";
import { useCssVarColor } from "@/hooks/useChartColor";

interface CalendarChartProps {
  data: {
    value: number;
    day: string;
  }[];
  start: Date;
  end: Date;
}

const CalendarTooltip: React.FC<CalendarTooltipProps> = ({ day, value }) => {
  const date = new Date(day);

  return (
    <div className="rounded-md bg-background p-2 text-xs shadow-md w-32">
      <div className="font-medium text-foreground">
        {format(date, "EEEE, MMM d")}
      </div>
      <div className="text-muted-foreground">
        {formatDuration(parseInt(value))}
      </div>
    </div>
  );
};

const CalendarChart: React.FC<CalendarChartProps> = ({ data, start, end }) => {
  const emptyColor = useCssVarColor("--input");
  const borderColor = useCssVarColor("--background");
  const monthTextColor = useCssVarColor("--muted-foreground");

  return (
    <div className="h-[200px] w-full">
      <ResponsiveCalendar
        data={data}
        from={start}
        to={end}
        emptyColor={emptyColor}
        colors={["#4ade80"]}
        margin={{ top: 40, right: 40, bottom: 40, left: 40 }}
        monthBorderColor={borderColor}
        dayBorderWidth={2}
        dayBorderColor={borderColor}
        tooltip={CalendarTooltip}
        legends={undefined}
        theme={{
          labels: {
            text: {
              fill: monthTextColor,
              fontSize: 12,
              fontWeight: 500,
            },
          },
        }}
      />
    </div>
  );
};

export default CalendarChart;
