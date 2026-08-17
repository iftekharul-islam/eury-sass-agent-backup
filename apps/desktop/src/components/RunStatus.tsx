import * as React from "react";
import { Icon } from "./Icons";
import type { RunRecord } from "../lib/session-store";

/** No event at all for this long and the run is reported as unanswered. */
const STALL_MS = 45_000;

/** How much of the reasoning stream stays on screen while it runs. */
const THINKING_TAIL_CHARS = 420;

function formatElapsed(ms: number): string {
  const seconds = Math.max(0, Math.floor(ms / 1000));
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${String(seconds % 60).padStart(2, "0")}s`;
}

function phaseLabel(run: RunRecord, stalled: boolean, toolName?: string): string {
  if (stalled && run.phase !== "awaiting_approval") return "waiting for the model…";
  switch (run.phase) {
    case "thinking":
      return "thinking…";
    case "responding":
      return "responding…";
    case "awaiting_approval":
      return toolName ? `waiting for your approval on ${toolName}…` : "waiting for your approval…";
    case "tool":
      return toolName ? `running ${toolName}…` : "running a tool…";
    default:
      return "starting…";
  }
}

/**
 * Token readout. The core's own count wins; until it reports one we show an
 * estimate from what has actually streamed, marked `~` so it is never mistaken
 * for the billed number.
 */
function tokenLabel(run: RunRecord): string | null {
  const reported = run.usage?.outputTokens;
  if (typeof reported === "number" && reported > 0) return `${reported} tokens`;
  if (run.streamedChars > 0) return `~${Math.max(1, Math.round(run.streamedChars / 4))} tokens`;
  return null;
}

export interface RunStatusProps {
  run: RunRecord;
  /** Reasoning streamed so far, shown live while the model is thinking. */
  thinking?: string;
  /** Name of the tool currently running, if any. */
  toolName?: string;
}

/**
 * The live "typing" state of a run: the reasoning as it streams, then a status
 * line with elapsed time, tokens and what the model is doing right now — so a
 * long run reads as progress instead of an indefinite spinner.
 *
 * Read-only by design: stopping a run is the composer's stop button (or Esc),
 * so the transcript does not repeat the control on every turn.
 */
export function RunStatus({ run, thinking, toolName }: RunStatusProps) {
  const [now, setNow] = React.useState(() => Date.now());
  const tailRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, [run.id]);

  // Keep the newest reasoning in view as it streams.
  React.useEffect(() => {
    const el = tailRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [thinking]);

  const stalled = now - run.lastEventAt > STALL_MS;
  const awaitingApproval = run.phase === "awaiting_approval";
  const tokens = tokenLabel(run);
  const tail = thinking?.trim().slice(-THINKING_TAIL_CHARS);

  return (
    <div className="run-status">
      {tail && (
        <div className="run-status-thinking" ref={tailRef} aria-live="polite">
          {tail}
        </div>
      )}

      <div className="run-status-line">
        <span
          className={`run-status-mark${stalled && !awaitingApproval ? " is-stalled" : ""}${awaitingApproval ? " is-awaiting-approval" : ""}`}
          aria-hidden="true"
        >
          <Icon name={awaitingApproval ? "alert" : "spark"} size={13} />
        </span>
        <span className="run-status-meta">
          {formatElapsed(now - run.startedAt)}
          {tokens ? ` · ${tokens}` : ""} · {phaseLabel(run, stalled, toolName)}
        </span>
      </div>

      {awaitingApproval && (
        <div className="run-status-hint">
          {toolName
            ? `${toolName} is blocked until you approve or deny below.`
            : "Approve or deny below to continue."}
        </div>
      )}

      {stalled && !awaitingApproval && (
        <div className="run-status-hint">
          Nothing has come back from the model for {formatElapsed(now - run.lastEventAt)}. Check
          that the API backend is running on port 3001, then try again — or press Esc to stop.
        </div>
      )}
    </div>
  );
}
