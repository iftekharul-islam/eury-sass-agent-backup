import { Icon } from "./Icons";

export interface TrustModalProps {
  isOpen: boolean;
  onClose: () => void;
  projectPath?: string;
  onTrust?: () => void;
  onOpenReadOnly?: () => void;
}

export function TrustModal({
  isOpen,
  onClose,
  projectPath = "",
  onTrust,
  onOpenReadOnly,
}: TrustModalProps) {
  if (!isOpen) return null;

  return (
    <div className="scrim" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
        <div className="mh">
          <Icon name="shield" size={20} style={{ color: "var(--color-warning)" }} />
          <h3>Trust this project?</h3>
        </div>
        <div className="mb">
          <div className="payload" style={{ marginBottom: "12px" }}>
            {projectPath}
          </div>
          <p>
            Until you trust it, Eury runs read-only in this project: no shell, no network, and no
            MCP servers. The project's own instructions and MCP configuration are shown for your
            review rather than applied.
          </p>
        </div>
        <div className="mf">
          <button type="button" className="btn ghost" onClick={onClose}>
            Cancel
          </button>
          <button
            type="button"
            className="btn primary"
            autoFocus
            onClick={() => {
              if (onOpenReadOnly) onOpenReadOnly();
              onClose();
            }}
          >
            Open read-only
          </button>
          <button
            type="button"
            className="btn"
            onClick={() => {
              if (onTrust) onTrust();
              onClose();
            }}
          >
            Trust project
          </button>
        </div>
      </div>
    </div>
  );
}
