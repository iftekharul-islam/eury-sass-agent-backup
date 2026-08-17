import * as React from "react";
import { cn } from "./utils";

export const Skeleton = React.forwardRef<HTMLDivElement, React.HTMLAttributes<HTMLDivElement>>(
  ({ className, ...props }, ref) => (
    <div
      ref={ref}
      className={cn("animate-pulse rounded-md bg-[var(--color-bg-hover)]", className)}
      {...props}
    />
  )
);
Skeleton.displayName = "Skeleton";
