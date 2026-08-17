import * as React from "react";
import { Icon } from "./Icons";
import {
  activityTarget,
  isWriteTool,
  selectRunActivities,
  selectRuns,
  useSessionState,
  type RunRecord,
  type RunStatus,
} from "../lib/session-store";

export interface RunsViewProps {
  workspaceRoot?: string;
  workspaceName?: string;
  onOpenConversation?: (conversationId: string) => void;
  onViewChanges?: () => void;
}

function statusIcon(status: RunStatus) {
  switch (status) {
    case "completed":
      return <Icon name="check" style={{ color: "var(--color-success)" }} />;
    case "failed":
      return <Icon name="alert" style={{ color: "var(--color-danger)" }} />;
    case "cancelled":
      return <Icon name="x" style={{ color: "var(--color-fg-subtle)" }} />;
    case "running":
      return <span className="spinner" style={{ width: "11px", height: "11px" }} />;
  }
}

function statusLabel(status: RunStatus): string {
  switch (status) {
    case "completed":
      return "Completed";
    case "failed":
      return "Failed";
    case "cancelled":
      return "Cancelled";
    case "running":
      return "Running";
  }
}

function formatDuration(run: RunRecord): string {
  const end = run.endedAt ?? Date.now();
  const seconds = Math.max(0, Math.round((end - run.startedAt) / 1000));
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${seconds % 60}s`;
}

function dayGroup(at: number): string {
  const day = new Date(at);
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(today.getDate() - 1);

  const sameDay = (a: Date, b: Date) => a.toDateString() === b.toDateString();
  if (sameDay(day, today)) return "Today";
  if (sameDay(day, yesterday)) return "Yesterday";
  return day.toLocaleDateString();
}

function runMeta(run: RunRecord, toolCount: number): string {
  const parts = [run.mode, run.modelLabel, `${toolCount} tool${toolCount === 1 ? "" : "s"}`, formatDuration(run)];
  if (run.usage?.inputTokens !== undefined || run.usage?.outputTokens !== undefined) {
    parts.push(
      `${((run.usage.inputTokens ?? 0) / 1000).toFixed(1)}k in / ${((run.usage.outputTokens ?? 0) / 1000).toFixed(1)}k out`,
    );
  }
  if (run.usage?.costUsd !== undefined) parts.push(`$${run.usage.costUsd.toFixed(2)}`);
  return parts.join(" · ");
}

export function RunsView({
  workspaceRoot,
  workspaceName,
  onOpenConversation,
  onViewChanges,
}: RunsViewProps) {
  const session = useSessionState();
  const runs = React.useMemo(() => selectRuns(session, workspaceRoot), [session, workspaceRoot]);

  const [selectedRunId, setSelectedRunId] = React.useState<string | null>(null);
  const selectedRun = runs.find((run) => run.id === selectedRunId) ?? runs[0];
  const activities = React.useMemo(
    () => (selectedRun ? selectRunActivities(session, selectedRun.id) : []),
    [session, selectedRun?.id],
  );
  const decisions = React.useMemo(
    () =>
      selectedRun
        ? session.approvals.filter((record) => record.runId === selectedRun.id && record.decidedAt)
        : [],
    [session.approvals, selectedRun?.id],
  );

  const groups = React.useMemo(() => {
    const map = new Map<string, RunRecord[]>();
    for (const run of runs) {
      const key = dayGroup(run.startedAt);
      map.set(key, [...(map.get(key) ?? []), run]);
    }
    return [...map.entries()];
  }, [runs]);

  return (
    <div className="main">
      <div className="pane-head">
        <h2>Runs</h2>
        <span className="sub">{workspaceName ?? "All workspaces"}</span>
        <span className="spacer" />
        <span className="hint">{runs.length} recorded</span>
      </div>

      {runs.length === 0 ? (
        <div style={{ padding: "60px 20px", textAlign: "center", color: "var(--color-fg-muted)" }}>
          <Icon name="activity" size={20} style={{ marginBottom: "8px" }} />
          <p style={{ margin: 0 }}>No runs yet. Send a message and it will be recorded here.</p>
        </div>
      ) : (
        <div className="split">
          <div className="split-left">
            {groups.map(([label, groupRuns]) => (
              <React.Fragment key={label}>
                <div className="sec-label">{label}</div>
                {groupRuns.map((run) => (
                  <div
                    key={run.id}
                    className={`row ${selectedRun?.id === run.id ? "on" : ""}`}
                    onClick={() => setSelectedRunId(run.id)}
                  >
                    {statusIcon(run.status)}
                    <span className="t">{run.title}</span>
                    <span className="m">{formatDuration(run)}</span>
                  </div>
                ))}
              </React.Fragment>
            ))}
          </div>

          <div className="split-right">
            <div className="inner">
              {selectedRun && (
                <>
                  <h2 style={{ margin: "0 0 6px" }}>{selectedRun.title}</h2>
                  <div className="actions" style={{ marginBottom: "18px" }}>
                    <span
                      className={`status ${
                        selectedRun.status === "completed"
                          ? "ok"
                          : selectedRun.status === "failed"
                          ? "err"
                          : "warn"
                      }`}
                    >
                      {statusIcon(selectedRun.status)}
                      {statusLabel(selectedRun.status)}
                    </span>
                    <span className="hint">{runMeta(selectedRun, activities.length)}</span>
                  </div>

                  {selectedRun.error && (
                    <div className="setting" style={{ color: "var(--color-danger)" }}>
                      <Icon name="alert" size={16} />
                      <div className="t">{selectedRun.error}</div>
                    </div>
                  )}

                  <div className="sec-label">Timeline</div>
                  {activities.length === 0 ? (
                    <div className="hint" style={{ padding: "8px 0" }}>
                      This run used no tools.
                    </div>
                  ) : (
                    activities.map((activity) => (
                      <div key={activity.id} className="tl">
                        <span className="at">
                          {`${((activity.startedAt - selectedRun.startedAt) / 1000).toFixed(1)}s`}
                        </span>
                        <Icon
                          name={isWriteTool(activity.name) ? "pencil" : "terminal"}
                          style={{ color: "var(--color-fg-subtle)" }}
                        />
                        <span className="t mono">{activityTarget(activity)}</span>
                        {typeof activity.plus === "number" && activity.plus > 0 && (
                          <span className="plus">+{activity.plus}</span>
                        )}
                        {typeof activity.minus === "number" && activity.minus > 0 && (
                          <span className="minus">−{activity.minus}</span>
                        )}
                        <span className={`status ${activity.status === "succeeded" ? "ok" : "err"}`}>
                          <Icon name={activity.status === "succeeded" ? "check" : "x"} />
                          {activity.status}
                        </span>
                        {activity.durationMs !== undefined && (
                          <span className="m">
                            {activity.durationMs < 1000
                              ? `${activity.durationMs}ms`
                              : `${(activity.durationMs / 1000).toFixed(1)}s`}
                          </span>
                        )}
                      </div>
                    ))
                  )}

                  {decisions.length > 0 && (
                    <>
                      <div className="sec-label" style={{ marginTop: "20px" }}>
                        Decisions
                      </div>
                      {decisions.map((record) => (
                        <div key={record.id} className="setting">
                          <Icon
                            name={record.decision === "denied" ? "x" : "check"}
                            size={16}
                            style={{
                              color:
                                record.decision === "denied"
                                  ? "var(--color-danger)"
                                  : "var(--color-success)",
                            }}
                          />
                          <div className="t">
                            {record.toolName} <span className="mono">{record.target}</span>
                            <div className="sub">
                              {record.decision === "denied"
                                ? "Denied"
                                : record.decision === "allow_session"
                                ? "Allowed for session"
                                : "Allowed once"}
                              {record.decidedAt
                                ? ` · ${((record.decidedAt - record.requestedAt) / 1000).toFixed(1)}s to decide`
                                : ""}
                            </div>
                          </div>
                        </div>
                      ))}
                    </>
                  )}

                  <div className="actions" style={{ marginTop: "24px" }}>
                    <button
                      type="button"
                      className="btn primary"
                      onClick={() => onOpenConversation?.(selectedRun.conversationId)}
                    >
                      Open conversation
                    </button>
                    <button type="button" className="btn" onClick={onViewChanges}>
                      View changes
                    </button>
                  </div>
                </>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
