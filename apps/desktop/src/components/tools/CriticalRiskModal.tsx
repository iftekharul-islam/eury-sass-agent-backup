import * as React from "react";
import { Icon } from "../Icons";

export interface CriticalRiskModalProps {
  isOpen: boolean;
  toolName: string;
  target: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function CriticalRiskModal({
  isOpen,
  toolName,
  target,
  onConfirm,
  onCancel,
}: CriticalRiskModalProps) {
  const [confirmText, setConfirmText] = React.useState("");

  React.useEffect(() => {
    if (!isOpen) {
      setConfirmText("");
    }
  }, [isOpen]);

  if (!isOpen) return null;

  // Real app would extract last path segment or command name. We'll just ask for the exact target for the stub.
  const expectedText = target.split("/").pop() || target;
  const isMatch = confirmText === expectedText;

  return (
    <div className="modal-overlay" style={{ zIndex: 1000 }}>
      <div className="modal-content" style={{ maxWidth: 450 }}>
        <div className="modal-header" style={{ borderBottom: "none", paddingBottom: 0 }}>
          <h2 style={{ display: "flex", alignItems: "center", gap: 8, color: "var(--color-danger)" }}>
            <Icon name="alert" /> Critical Risk Action
          </h2>
          <button className="btn sm icon ghost" onClick={onCancel}>
            <Icon name="x" />
          </button>
        </div>
        <div className="modal-body" style={{ paddingTop: 12 }}>
          <p style={{ marginBottom: 16 }}>
            The agent has requested to execute a critical action. Are you sure you want to allow this?
          </p>
          <div className="payload" style={{ marginBottom: 16, background: "var(--color-bg-sunken)", padding: 12, borderRadius: 6 }}>
            <div style={{ fontWeight: 500, marginBottom: 4 }}>{toolName}</div>
            <div className="mono" style={{ fontSize: 12, color: "var(--color-fg-muted)" }}>{target}</div>
          </div>
          <p style={{ fontSize: 13, marginBottom: 8 }}>
            Type <span className="mono">{expectedText}</span> to confirm.
          </p>
          <input
            autoFocus
            type="text"
            className="input"
            value={confirmText}
            onChange={(e) => setConfirmText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && isMatch) onConfirm();
              if (e.key === "Escape") onCancel();
            }}
          />
        </div>
        <div className="modal-footer" style={{ borderTop: "none", paddingTop: 0, justifyContent: "space-between" }}>
          <span className="hint">This action cannot be undone.</span>
          <div style={{ display: "flex", gap: 8 }}>
            <button className="btn" onClick={onCancel}>Cancel</button>
            <button
              className="btn danger"
              disabled={!isMatch}
              onClick={onConfirm}
            >
              Confirm and Allow
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
