import { cn } from "@skopio/ui";

interface BorderBeamProps {
  loading: boolean;
  className?: string;
}

const BorderBeam: React.FC<BorderBeamProps> = ({ loading, className }) => {
  if (!loading) return null;

  return (
    <div
      style={
        {
          "--border-width": 2.5,
          "--size": 300,
          "--color-from": "#c111ed",
          "--color-to": "#e36e0e",
          "--delay": "0s",
          "--anchor": 70,
          "--duration": 7,
        } as React.CSSProperties
      }
      className={cn(
        "pointer-events-none absolute inset-[0] rounded-[inherit] [border:calc(var(--border-width)*1px)_solid_transparent]",

        // mask styles
        "![mask-clip:padding-box,border-box] ![mask-composite:intersect] [mask:linear-gradient(transparent,transparent),linear-gradient(white,white)]",

        // pseudo styles
        "after:absolute after:aspect-square after:w-[calc(var(--size)*1px)] after:[animation-delay:var(--delay)] after:[background:linear-gradient(to_left,var(--color-from),var(--color-to),transparent)] after:[offset-anchor:calc(var(--anchor)*1%)_50%] after:[offset-path:rect(0_auto_auto_0_round_calc(var(--size)*1px))]",

        // Animation class - needs to be defined in your CSS
        "after:animate-[border-beam_calc(var(--duration)*1s)_infinite_linear]",

        className,
      )}
    />
  );
};

export default BorderBeam;
