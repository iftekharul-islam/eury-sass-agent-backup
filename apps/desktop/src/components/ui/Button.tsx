import * as React from "react";
import { cn } from "./utils";

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "secondary" | "subtle" | "danger" | "ghost" | "icon";
  size?: "default" | "sm" | "icon";
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "secondary", size = "default", ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={cn(
          "inline-flex items-center justify-center whitespace-nowrap rounded-md text-[12.5px] font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)] disabled:pointer-events-none disabled:opacity-50",
          {
            "bg-[var(--color-accent)] text-[var(--color-accent-fg)] hover:bg-[var(--color-accent-hover)] shadow-sm": variant === "primary",
            "bg-[var(--color-bg-elevated)] border border-[var(--color-border)] text-[var(--color-fg)] hover:bg-[var(--color-bg-hover)] shadow-sm": variant === "secondary",
            "bg-transparent text-[var(--color-fg-muted)] hover:bg-[var(--color-bg-hover)] hover:text-[var(--color-fg)]": variant === "subtle" || variant === "ghost",
            "bg-[var(--color-danger)] text-white hover:opacity-90": variant === "danger",
            "bg-transparent hover:bg-[var(--color-bg-hover)] text-[var(--color-fg-muted)] hover:text-[var(--color-fg)]": variant === "icon",
            "h-8 px-3": size === "default" && variant !== "icon",
            "h-7 px-2 text-xs": size === "sm",
            "h-8 w-8": size === "icon" || variant === "icon",
          },
          className
        )}
        {...props}
      />
    );
  }
);
Button.displayName = "Button";
