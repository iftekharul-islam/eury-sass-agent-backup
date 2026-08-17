import * as React from "react";
import { Icon } from "./Icons";
import { ipcClient } from "../lib/ipc";
import {
  selectDecidedApprovals,
  selectPendingApprovals,
  sessionStore,
  useSessionState,
  type ApprovalDecision,
  type ApprovalRecord,
} from "../lib/session-store";

export interface ApprovalsViewProps {
  workspaceRoot?: string;
  onBack?: () => void;
}

function relativeTime(ts: number): string {
  const diff = Date.now() - ts;
  if (diff < 60_000) return "Just now";
  if (diff < 3_600_000) return `${Math.round(diff / 60_000)}m ago`;
  if (diff < 86_400_000) return `${Math.round(diff / 3_600_000)}h ago`;
  return `${Math.round(diff / 86_400_000)}d ago`;
}

function decisionLabel(decision?: ApprovalDecision): string {
  switch (decision) {
    case "denied":
      return "Denied";
    case "allow_session":
      return "Allowed for session";
    case "allow_once":
      return "Allowed once";
    default:
      return "Pending";
  }
}

export function ApprovalsView({ workspaceRoot, onBack }: ApprovalsViewProps) {
  const session = useSessionState();
  const pending = React.useMemo<ApprovalRecord[]>(
    () => selectPendingApprovals(session, workspaceRoot),
    [session, workspaceRoot],
  );
  const history = React.useMemo<ApprovalRecord[]>(
    () => selectDecidedApprovals(session, workspaceRoot),
    [session, workspaceRoot],
  );

  const [activeTab, setActiveTab] = React.useState<"pending" | "history">(
    pending.length > 0 ? "pending" : "history",
  );

  const decide = (record: ApprovalRecord, decision: ApprovalDecision) => {
    sessionStore.recordApprovalDecision(record.id, decision);
    void ipcClient.run.approve(record.id, decision).catch((err) => {
      console.error("Failed to send approval decision:", err);
    });
  };

  return (
    <div className="center-pane">
      <div className="titlebar-shim" />

      <div className="pane-header">
        <div className="left">
          {onBack && (
            <button className="btn icon ghost" onClick={onBack}>
              <Icon name="chev-r" style={{ transform: "rotate(180deg)" }} />
            </button>
          )}
          <Icon name="alert" />
          <span className="title">Approvals</span>
        </div>
        <div className="right">
          <div className="tabs">
            <button
              className={`tab ${activeTab === "pending" ? "on" : ""}`}
              onClick={() => setActiveTab("pending")}
            >
              Pending
              {pending.length > 0 && <span className="count">{pending.length}</span>}
            </button>
            <button
              className={`tab ${activeTab === "history" ? "on" : ""}`}
              onClick={() => setActiveTab("history")}
            >
              History
            </button>
          </div>
        </div>
      </div>

      <div className="pane-content" style={{ padding: "24px" }}>
        {activeTab === "pending" &&
          (pending.length === 0 ? (
            <div style={{ color: "var(--color-fg-muted)", fontSize: "13px" }}>
              Nothing is waiting on you.
            </div>
          ) : (
            <div className="approvals-list">
              <h3 style={{ marginBottom: "16px", color: "var(--color-fg-muted)" }}>
                Awaiting your decision
              </h3>
              {pending.map((record) => (
                <div
                  key={record.id}
                  className="card"
                  style={{
                    marginBottom: "12px",
                    padding: "16px",
                    border: "1px solid var(--color-border-strong)",
                    borderRadius: "var(--r-md)",
                  }}
                >
                  <div
                    style={{ display: "flex", justifyContent: "space-between", marginBottom: "12px" }}
                  >
                    <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                      <Icon name="terminal" />
                      <strong>{record.toolName}</strong>
                    </div>
                    <span className="hint">{relativeTime(record.requestedAt)}</span>
                  </div>
                  <div
                    className="mono"
                    style={{
                      background: "var(--color-bg-inset)",
                      padding: "8px",
                      borderRadius: "var(--r-sm)",
                      marginBottom: "12px",
                    }}
                  >
                    {record.target}
                  </div>
                  <div style={{ display: "flex", gap: "8px" }}>
                    <button
                      className="btn primary sm"
                      onClick={() => decide(record, "allow_once")}
                    >
                      Allow once
                    </button>
                    <button className="btn sm" onClick={() => decide(record, "allow_session")}>
                      Allow for session
                    </button>
                    <button className="btn sm" onClick={() => decide(record, "denied")}>
                      Deny
                    </button>
                  </div>
                </div>
              ))}
            </div>
          ))}

        {activeTab === "history" &&
          (history.length === 0 ? (
            <div style={{ color: "var(--color-fg-muted)", fontSize: "13px" }}>
              No approval decisions recorded yet.
            </div>
          ) : (
            <div className="approvals-list">
              <h3 style={{ color: "var(--color-fg-muted)", marginBottom: "16px" }}>
                Recent decisions
              </h3>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "13px" }}>
                <thead>
                  <tr
                    style={{
                      borderBottom: "1px solid var(--color-border)",
                      color: "var(--color-fg-muted)",
                      textAlign: "left",
                    }}
                  >
                    <th style={{ padding: "8px 0", fontWeight: "normal" }}>Action</th>
                    <th style={{ padding: "8px 0", fontWeight: "normal" }}>Target</th>
                    <th style={{ padding: "8px 0", fontWeight: "normal" }}>Decision</th>
                    <th style={{ padding: "8px 0", fontWeight: "normal" }}>Time</th>
                  </tr>
                </thead>
                <tbody>
                  {history.map((record) => (
                    <tr
                      key={`${record.id}-${record.decidedAt}`}
                      style={{ borderBottom: "1px solid var(--color-border-subtle)" }}
                    >
                      <td
                        style={{
                          padding: "12px 0",
                          display: "flex",
                          alignItems: "center",
                          gap: "8px",
                        }}
                      >
                        <Icon name={record.toolName === "run_command" ? "terminal" : "pencil"} size={14} />
                        {record.toolName}
                      </td>
                      <td style={{ padding: "12px 0", fontFamily: "var(--font-mono)" }}>
                        {record.target}
                      </td>
                      <td style={{ padding: "12px 0" }}>
                        <span
                          className={`status ${record.decision === "denied" ? "err" : "ok"}`}
                        >
                          {decisionLabel(record.decision)}
                        </span>
                      </td>
                      <td style={{ padding: "12px 0", color: "var(--color-fg-muted)" }}>
                        {record.decidedAt ? relativeTime(record.decidedAt) : ""}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ))}
      </div>
    </div>
  );
}
