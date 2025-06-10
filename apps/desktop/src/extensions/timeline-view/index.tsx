import { TimelineBin } from "./TimelineView";
import { TimelineView } from "./TimelineView";

const mockData: TimelineBin[] = Array.from({ length: 200 }, (_, i) => {
  const timestamp = Date.now() - i * 15 * 60 * 1000;
  return {
    timestamp,
    afk: Math.random(),
    vscode: Math.random(),
    general: Math.random(),
  };
}).reverse();

const TimelineExtension = () => {
  return <TimelineView data={mockData} />;
};

export default TimelineExtension;
