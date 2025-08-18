import { Slider } from "@skopio/ui";

interface SettingsContentProps {
  title: string;
  topN: number;
  setTopN: (val: number) => void;
}

export const MIN_TOP_N = 2;
const MAX_TOP_N = 20;

const SettingsContent: React.FC<SettingsContentProps> = ({
  title,
  topN,
  setTopN,
}) => {
  return (
    <div className="w-64">
      <div className="mb-2 text-sm font-medium text-muted-foreground">
        Show top N {title}
      </div>
      <div className="flex items-center gap-3">
        <Slider
          value={[topN]}
          min={MIN_TOP_N}
          max={MAX_TOP_N}
          step={1}
          onValueChange={(val) => setTopN(val[0] ?? MIN_TOP_N)}
          aria-label="Top N"
          className="flex-1"
        />
        <p className="w-10 text-right text-sm tabular-nums">{topN}</p>
      </div>
      <p className="mt-2 text-xs text-muted-foreground">
        Only the top {topN} series are shown. The rest are grouped into "Other".
      </p>
    </div>
  );
};

export default SettingsContent;
