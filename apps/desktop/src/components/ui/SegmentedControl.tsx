import * as React from "react";
import { cn } from "./utils";

interface SegmentedControlProps {
  options: { label: string; value: string }[];
  value: string;
  onChange: (value: string) => void;
  className?: string;
}

export function SegmentedControl({ options, value, onChange, className }: SegmentedControlProps) {
  return (
    <div
      className={cn(
        "inline-flex h-7 items-center justify-center rounded-md bg-[var(--color-bg-inset)] p-1 text-[var(--color-fg-muted)]",
        className
      )}
      role="radiogroup"
    >
      {options.map((opt) => {
        const isSelected = value === opt.value;
        return (
          <button
            key={opt.value}
            role="radio"
            aria-checked={isSelected}
            onClick={() => onChange(opt.value)}
            className={cn(
              "inline-flex items-center justify-center whitespace-nowrap rounded-sm px-3 py-1 text-xs font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)]",
              isSelected
                ? "bg-[var(--color-bg-elevated)] text-[var(--color-fg)] shadow-sm border border-[var(--color-border)]"
                : "hover:text-[var(--color-fg)] hover:bg-[var(--color-bg-hover)]"
            )}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}
