import * as React from "react";
import { Icon } from "../Icons";
import { activityTarget, type ToolActivityRecord } from "../../lib/session-store";

/**
 * How long the buttons stay inert after the card appears. An approval can
 * arrive while the user is mid-keystroke, and a write or a shell command is
 * not something to grant by accident.
 */
const ARM_DELAY_MS = 400;

export interface ApprovalCardProps {
  activity: ToolActivityRecord;
  onAllowOnce: () => void;
  onAllowSession?: () => void;
  onDeny: () => void;
  isWrite?: boolean;
}

export function ApprovalCard({
  activity,
  onAllowOnce,
  onAllowSession,
  onDeny,
  isWrite,
}: ApprovalCardProps) {
  const [armed, setArmed] = React.useState(false);
  const [showMenu, setShowMenu] = React.useState(false);
  const allowRef = React.useRef<HTMLButtonElement>(null);
  const menuRef = React.useRef<HTMLDivElement>(null);

  React.useEffect(() => {
    const timer = setTimeout(() => setArmed(true), ARM_DELAY_MS);
    return () => clearTimeout(timer);
  }, []);

  // The run is blocked on this card, so it takes the focus: Enter approves and
  // the decision needs no reach for the mouse.
  React.useEffect(() => {
    if (armed) allowRef.current?.focus();
  }, [armed]);

  React.useEffect(() => {
    if (!showMenu) return;
    const onPointerDown = (event: PointerEvent) => {
      if (!menuRef.current?.contains(event.target as Node)) setShowMenu(false);
    };
    window.addEventListener("pointerdown", onPointerDown);
    return () => window.removeEventListener("pointerdown", onPointerDown);
  }, [showMenu]);

  const isElevated = activity.name === "run_command";
  const riskClass = isElevated ? "elevated" : "medium";
  const riskLabel = isElevated ? "Elevated risk" : "Medium risk";

  const target = activityTarget(activity);
  const writePreview =
    activity.diff ??
    (typeof activity.payload?.content === "string" ? activity.payload.content : undefined);
  const justification =
    typeof activity.payload?.justification === "string" ? activity.payload.justification : null;

  return (
    <div className="card approve" role="group" aria-label="Approval required">
      <div className="ap-head">
        <Icon name="alert" />
        <span>Approval required</span>
        <span className="spacer" />
        <span className={`risk ${riskClass}`}>{riskLabel}</span>
      </div>
      <div className="card-body">
        {isWrite ? (
          <div className="diff approve-target">
            <div className="diff-head">
              <Icon name="pencil" style={{ color: "var(--color-fg-muted)" }} />
              <span className="path">{target}</span>
            </div>
            <div className="diff-body approve-preview">
              <pre>{writePreview ?? "The agent has not sent a preview for this write."}</pre>
            </div>
          </div>
        ) : (
          <div className="approve-command">
            <Icon name="terminal" size={13} />
            <code>{target}</code>
          </div>
        )}

        {justification && (
          <dl className="meta">
            <dt>Reason</dt>
            <dd>{justification}</dd>
          </dl>
        )}

        <div className="actions">
          <button type="button" className="btn" onClick={onDeny}>
            Deny
          </button>

          <button
            type="button"
            className="btn primary focusring"
            ref={allowRef}
            disabled={!armed}
            onClick={onAllowOnce}
          >
            {isWrite ? "Apply once" : "Allow once"}
          </button>

          {onAllowSession && (
            <div className="approve-menu" ref={menuRef}>
              <button
                type="button"
                className="btn withmenu"
                disabled={!armed}
                aria-expanded={showMenu}
                onClick={() => setShowMenu((open) => !open)}
              >
                {isWrite ? "Allow edits in this file" : "Allow for this session"}
                <span className="div" />
                <Icon name="chev-d" />
              </button>
              {showMenu && (
                <div className="approve-menu-pop" role="menu">
                  <button type="button" className="row" onClick={onAllowSession}>
                    Allow for session
                  </button>
                </div>
              )}
            </div>
          )}

          <span className="spacer" />
          <span className="hint">Esc stops the run</span>
        </div>
      </div>
    </div>
  );
}
