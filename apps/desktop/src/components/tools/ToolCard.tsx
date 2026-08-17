import React from "react";
import { Icon } from "../Icons";
import { activityTarget, isWriteTool, type ToolActivityRecord } from "../../lib/session-store";

export type ToolActivity = ToolActivityRecord;

export interface ToolCardProps {
  activity: ToolActivityRecord;
  /** Rendered next to the status when the tool wrote to disk. */
  onViewDiff?: (activity: ToolActivityRecord) => void;
}

function iconFor(name: string): string {
  switch (name) {
    case "run_command":
    case "terminal":
      return "terminal";
    case "read_file":
    case "list_dir":
    case "glob":
    case "grep":
      return "file";
    case "write_file":
    case "edit_file":
    case "apply_patch":
    case "create_file":
    case "delete_file":
      return "pencil";
    default:
      return "tool";
  }
}

export function ToolCard({ activity, onViewDiff }: ToolCardProps) {
  const isRunning = activity.status === "running";
  const isFailed = activity.status === "failed";
  const isDenied = activity.status === "denied";
  const isSucceeded = activity.status === "succeeded";
  const isCommand = activity.name === "run_command";
  const hasBody = Boolean(activity.stdout || activity.stderr || activity.diff);
  const [expanded, setExpanded] = React.useState(false);
  const outputRef = React.useRef<HTMLDivElement>(null);

  // A finished call collapses; a running or failed one stays open so the user
  // sees the output as it lands.
  const showBody =
    (hasBody || (isRunning && isCommand)) && (expanded || isRunning || isFailed);

  React.useEffect(() => {
    if (!isRunning || !outputRef.current) return;
    outputRef.current.scrollTop = outputRef.current.scrollHeight;
  }, [isRunning, activity.stdout, activity.stderr]);

  return (
    <div className="card">
      <div
        className="card-head"
        style={{ cursor: hasBody || (isRunning && isCommand) ? "pointer" : "default" }}
        onClick={() => (hasBody || (isRunning && isCommand)) && setExpanded((value) => !value)}
      >
        <Icon name={showBody ? "chev-d" : "chev-r"} className="ico" />
        <Icon name={iconFor(activity.name)} className="ico" />
        <span className="name">{activity.name}</span>
        <span className="target">{activityTarget(activity)}</span>
        <span className="spacer" />

        {typeof activity.plus === "number" && activity.plus > 0 && (
          <span className="plus">+{activity.plus}</span>
        )}
        {typeof activity.minus === "number" && activity.minus > 0 && (
          <span className="minus">−{activity.minus}</span>
        )}

        {activity.durationMs !== undefined && (
          <span className="dur">
            {activity.durationMs < 1000
              ? `${activity.durationMs}ms`
              : `${(activity.durationMs / 1000).toFixed(1)}s`}
          </span>
        )}

        {isSucceeded && isWriteTool(activity.name) && onViewDiff && (
          <button
            type="button"
            className="btn sm ghost"
            onClick={(e) => {
              e.stopPropagation();
              onViewDiff(activity);
            }}
          >
            <Icon name="diff" />
            View diff
          </button>
        )}

        {isRunning && (
          <span className="status run">
            <span className="spinner" style={{ width: "11px", height: "11px" }} />
            Running
          </span>
        )}

        {isFailed && (
          <span className="status err">
            <Icon name="x" />
            Failed
          </span>
        )}

        {isDenied && (
          <span className="status err">
            <Icon name="x" />
            Denied
          </span>
        )}

        {isSucceeded && (
          <span className="status ok">
            <Icon name="check" />
            {isWriteTool(activity.name) ? "Applied" : "Completed"}
          </span>
        )}
      </div>

      {showBody && (
        <div className="card-body">
          {activity.diff && (
            <div className="diff" style={{ marginTop: 8 }}>
              <pre className="diff-body" style={{ margin: 0, padding: 8, fontSize: 12 }}>
                {activity.diff}
              </pre>
            </div>
          )}

          {(activity.stdout || activity.stderr || (isRunning && isCommand)) && (
            <div
              ref={outputRef}
              className={`output${isRunning ? " is-live" : ""}`}
              style={{ marginTop: 8 }}
            >
              {activity.stdout ? <div>{activity.stdout}</div> : null}
              {activity.stderr ? <div className="fail">{activity.stderr}</div> : null}
              {isRunning && !activity.stdout && !activity.stderr ? (
                <div className="dimline">Waiting for output…</div>
              ) : null}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
