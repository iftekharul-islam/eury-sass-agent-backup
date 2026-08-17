import * as React from "react";
import { cn } from "./utils";

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "success" | "warning" | "danger" | "info" | "outline";
}

export function Badge({ className, variant = "default", ...props }: BadgeProps) {
  return (
    <div
      className={cn(
        "inline-flex items-center rounded-[var(--radius-sm)] px-2 py-0.5 text-[11px] font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)] focus:ring-offset-2",
        {
          "bg-[var(--color-bg-hover)] text-[var(--color-fg)]": variant === "default",
          "bg-[var(--color-success)] text-white": variant === "success",
          "bg-[var(--color-warning)] text-white": variant === "warning",
          "bg-[var(--color-danger)] text-white": variant === "danger",
          "bg-[var(--color-info)] text-white": variant === "info",
          "border border-[var(--color-border)] text-[var(--color-fg-muted)]": variant === "outline",
        },
        className
      )}
      {...props}
    />
  );
}

export function StatusIndicator({ variant, className, label }: { variant: BadgeProps["variant"], className?: string, label?: string }) {
  return (
    <div className={cn("flex items-center gap-1.5", className)}>
      <span
        className={cn("h-2 w-2 rounded-full", {
          "bg-[var(--color-fg-subtle)]": variant === "default",
          "bg-[var(--color-success)]": variant === "success",
          "bg-[var(--color-warning)]": variant === "warning",
          "bg-[var(--color-danger)]": variant === "danger",
          "bg-[var(--color-info)]": variant === "info",
        })}
      />
      {label && <span className="text-[11.5px] text-[var(--color-fg-muted)]">{label}</span>}
    </div>
  );
}
