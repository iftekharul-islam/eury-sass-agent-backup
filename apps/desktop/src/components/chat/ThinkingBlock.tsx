import * as React from "react";
import { Icon } from "../Icons";

/** How much of the reasoning stays on screen while it streams. */
const LIVE_TAIL_CHARS = 600;

function formatDuration(ms: number): string {
  const seconds = ms / 1000;
  if (seconds < 1) return "less than a second";
  if (seconds < 60) return `${seconds < 10 ? seconds.toFixed(1) : Math.round(seconds)}s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${String(Math.round(seconds % 60)).padStart(2, "0")}s`;
}

export interface ThinkingBlockProps {
  /** Reasoning streamed so far. */
  text: string;
  /** True while deltas are still arriving. */
  live: boolean;
  /** Time spent reasoning, once the window has closed. */
  durationMs?: number;
}

/**
 * The model's reasoning, in the two states it actually has: streaming — muted,
 * unfolded, tailing the newest text — and finished, folded into a single
 * "Thought for 8s" line the user can open again.
 *
 * The fold is automatic: reasoning is scaffolding for the answer, so once the
 * answer starts it stops competing with it. A manual toggle wins from then on.
 */
export function ThinkingBlock({ text, live, durationMs }: ThinkingBlockProps) {
  // `null` means "follow the run"; a boolean is the user's own choice.
  const [manual, setManual] = React.useState<boolean | null>(null);
  const tailRef = React.useRef<HTMLDivElement>(null);
  const expanded = manual ?? live;

  // A new run resets the fold, so the next turn's reasoning is not stuck open
  // (or shut) because of a decision made on the previous one.
  React.useEffect(() => {
    if (live) setManual(null);
  }, [live]);

  // Keep the newest reasoning in view while it streams.
  React.useEffect(() => {
    if (!live) return;
    const el = tailRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [text, live]);

  const trimmed = text.trim();
  if (!trimmed && !live && !(durationMs && durationMs > 0)) return null;

  const body = live && trimmed ? trimmed.slice(-LIVE_TAIL_CHARS) : trimmed;
  const label = live
    ? "Thinking"
    : durationMs && durationMs > 0
      ? `Thought for ${formatDuration(durationMs)}`
      : "Thought process";

  return (
    <div className={`thinking${live ? " is-live" : ""}`}>
      <button
        type="button"
        className="thinking-toggle"
        aria-expanded={expanded}
        onClick={() => setManual(!expanded)}
      >
        <Icon name={expanded ? "chev-d" : "chev-r"} size={12} />
        <span className={live ? "thinking-label is-live" : "thinking-label"}>{label}</span>
      </button>

      <div className="thinking-reveal" data-open={expanded ? "true" : "false"}>
        <div className="thinking-reveal-inner">
          {body ? (
            <div
              className={`thinking-text${live ? " is-live" : ""}`}
              ref={tailRef}
              aria-live={live ? "polite" : "off"}
            >
              {body}
            </div>
          ) : live ? (
            <div className="thinking-text is-live thinking-placeholder" aria-live="polite">
              Working through your request…
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
