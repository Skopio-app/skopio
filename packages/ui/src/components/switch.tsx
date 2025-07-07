"use client";

import * as React from "react";
import * as SwitchPrimitive from "@radix-ui/react-switch";

import { cn } from "../utils/cn";

function Switch({
  className,
  ...props
}: React.ComponentProps<typeof SwitchPrimitive.Root>) {
  return (
    <SwitchPrimitive.Root
      className={cn(
        "relative h-[25px] w-[42px] cursor-default rounded-full bg-gray-300 outline-none ring-1 ring-gray-400 data-[state=checked]:bg-black",
        className,
      )}
      {...props}
      style={{ WebkitTapHighlightColor: "rgba(0, 0, 0, 0)" } as any}
    >
      <SwitchPrimitive.Thumb className="block size-[21px] translate-x-0.5 rounded-full bg-white transition-transform duration-100 will-change-transform data-[state=checked]:translate-x-[19px]" />
    </SwitchPrimitive.Root>
  );
}

export { Switch };
