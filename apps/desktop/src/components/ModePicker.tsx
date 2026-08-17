import * as React from "react";
import { createPortal } from "react-dom";
import { Icon } from "./Icons";

export interface ModeOption {
  id: string;
  label: string;
  description: string;
  /** Whether this mode may change files or run commands. */
  canEdit: boolean;
}

/**
 * The run modes the core actually supports — one per `RunMode` variant in
 * `agent-types`. Anything listed here has to survive the round trip through
 * `composerModeToRunMode`, so this list stays in lockstep with that enum.
 */
export const COMPOSER_MODES: ModeOption[] = [
  {
    id: "Agent",
    canEdit: true,
    label: "Agent",
    description: "Reads and edits the project, asking before risky steps.",
  },
  {
    id: "Plan",
    canEdit: false,
    label: "Plan",
    description: "Writes a step-by-step plan first. Changes nothing.",
  },
  {
    id: "Ask",
    canEdit: false,
    label: "Ask",
    description: "Answers questions about the code. Can run diagnostic commands.",
  },
  {
    id: "Build",
    canEdit: true,
    label: "Build",
    description: "Carries out an approved plan, step by step.",
  },
];

export interface ModePickerProps {
  value: string;
  onChange: (mode: string) => void;
  disabled?: boolean;
}

const PANEL_WIDTH = 268;

/**
 * The mode menu renders through a portal at fixed coordinates. Anchored
 * absolutely it was clipped by the composer's `overflow: hidden` ancestors,
 * which is what cut the list down to a sliver of its last two entries.
 */
export function ModePicker({ value, onChange, disabled }: ModePickerProps) {
  const [open, setOpen] = React.useState(false);
  const [layout, setLayout] = React.useState<{ top: number; left: number } | null>(null);
  const anchorRef = React.useRef<HTMLSpanElement>(null);
  const menuRef = React.useRef<HTMLDivElement>(null);

  const updateLayout = React.useCallback(() => {
    const anchor = anchorRef.current?.getBoundingClientRect();
    if (!anchor) return;
    const menuHeight = menuRef.current?.offsetHeight ?? 320;
    const gap = 8;
    const margin = 12;

    let left = anchor.right - PANEL_WIDTH;
    left = Math.max(margin, Math.min(left, window.innerWidth - PANEL_WIDTH - margin));

    // Opens upward from the composer; flips below only if it would not fit.
    let top = anchor.top - menuHeight - gap;
    if (top < margin) {
      top = Math.min(anchor.bottom + gap, window.innerHeight - menuHeight - margin);
    }
    setLayout({ top, left });
  }, []);

  React.useLayoutEffect(() => {
    if (!open) {
      setLayout(null);
      return;
    }
    updateLayout();
    const menuEl = menuRef.current;
    if (!menuEl) return;
    const observer = new ResizeObserver(() => updateLayout());
    observer.observe(menuEl);
    return () => observer.disconnect();
  }, [open, updateLayout]);

  React.useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (anchorRef.current?.contains(target) || menuRef.current?.contains(target)) return;
      setOpen(false);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", updateLayout);
    window.addEventListener("scroll", updateLayout, true);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", updateLayout);
      window.removeEventListener("scroll", updateLayout, true);
    };
  }, [open, updateLayout]);

  const active = COMPOSER_MODES.find((option) => option.id === value);

  const menu =
    open && typeof document !== "undefined"
      ? createPortal(
          <div
            ref={menuRef}
            className="mode-picker-menu"
            role="listbox"
            aria-label="Run mode"
            style={{
              position: "fixed",
              top: layout?.top ?? -9999,
              left: layout?.left ?? -9999,
              width: PANEL_WIDTH,
              zIndex: 300,
              visibility: layout ? "visible" : "hidden",
            }}
          >
            {COMPOSER_MODES.map((option) => {
              const selected = option.id === value;
              return (
                <button
                  key={option.id}
                  type="button"
                  role="option"
                  aria-selected={selected}
                  className={`mode-picker-item${selected ? " selected" : ""}`}
                  onClick={() => {
                    onChange(option.id);
                    setOpen(false);
                  }}
                >
                  <span className="mode-picker-item-main">
                    <span className="mode-picker-item-label">{option.label}</span>
                    {selected && <Icon name="check" size={13} />}
                  </span>
                  <span className="mode-picker-item-desc">{option.description}</span>
                </button>
              );
            })}
          </div>,
          document.body,
        )
      : null;

  return (
    <>
      <span
        ref={anchorRef}
        className={`select ${disabled ? "disabled" : ""}`}
        role="button"
        tabIndex={disabled ? -1 : 0}
        aria-haspopup="listbox"
        aria-expanded={open}
        onClick={() => !disabled && setOpen((prev) => !prev)}
        onKeyDown={(e) => {
          if (disabled) return;
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setOpen((prev) => !prev);
          }
        }}
      >
        {active?.label ?? value}
        <Icon name="chev-d" />
      </span>
      {menu}
    </>
  );
}
