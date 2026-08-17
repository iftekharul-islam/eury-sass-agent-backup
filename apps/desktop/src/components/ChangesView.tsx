import * as React from "react";
import { Icon } from "./Icons";
import { selectChangedFiles, useSessionState, type ChangedFile } from "../lib/session-store";

export interface ChangesViewProps {
  workspaceRoot?: string;
  workspaceName?: string;
  onBack?: () => void;
  onOpenRun?: () => void;
  onOpenInEditor?: (path: string) => void;
}

interface DiffLine {
  type: "ctx" | "add" | "del" | "meta";
  oldLine?: number;
  newLine?: number;
  content: string;
}

/** Turns a unified diff into numbered lines for the viewer. */
function parseUnifiedDiff(diff: string): DiffLine[] {
  const lines: DiffLine[] = [];
  let oldLine = 0;
  let newLine = 0;

  for (const raw of diff.split("\n")) {
    if (raw.startsWith("@@")) {
      const match = /@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(raw);
      if (match) {
        oldLine = Number(match[1]);
        newLine = Number(match[2]);
      }
      lines.push({ type: "meta", content: raw });
      continue;
    }
    if (raw.startsWith("+++") || raw.startsWith("---") || raw.startsWith("diff ") || raw.startsWith("index ")) {
      lines.push({ type: "meta", content: raw });
      continue;
    }
    if (raw.startsWith("+")) {
      lines.push({ type: "add", newLine: newLine++, content: raw.slice(1) });
      continue;
    }
    if (raw.startsWith("-")) {
      lines.push({ type: "del", oldLine: oldLine++, content: raw.slice(1) });
      continue;
    }
    lines.push({
      type: "ctx",
      oldLine: oldLine++,
      newLine: newLine++,
      content: raw.startsWith(" ") ? raw.slice(1) : raw,
    });
  }

  return lines;
}

function EmptyChanges({ workspaceRoot }: { workspaceRoot?: string }) {
  return (
    <div style={{ padding: "60px 20px", textAlign: "center", color: "var(--color-fg-muted)" }}>
      <Icon name="file" size={20} style={{ marginBottom: "8px" }} />
      <p style={{ margin: 0 }}>
        {workspaceRoot
          ? "No files have been changed in this workspace yet."
          : "Open a project folder to track file changes."}
      </p>
    </div>
  );
}

export function ChangesView({
  workspaceRoot,
  workspaceName,
  onBack,
  onOpenRun,
  onOpenInEditor,
}: ChangesViewProps) {
  const session = useSessionState();
  const files = React.useMemo<ChangedFile[]>(
    () => selectChangedFiles(session, workspaceRoot),
    [session, workspaceRoot],
  );

  const [selectedPath, setSelectedPath] = React.useState<string | null>(null);
  const [diffMode, setDiffMode] = React.useState<"unified" | "split">("unified");

  const selectedFile = files.find((f) => f.path === selectedPath) ?? files[0];
  const totalPlus = files.reduce((acc, f) => acc + f.plus, 0);
  const totalMinus = files.reduce((acc, f) => acc + f.minus, 0);

  const diffLines = React.useMemo(
    () => (selectedFile?.diff ? parseUnifiedDiff(selectedFile.diff) : []),
    [selectedFile?.diff],
  );

  return (
    <div className="main">
      <div className="pane-head">
        {onBack && (
          <button type="button" className="btn sm ghost" onClick={onBack}>
            <Icon name="chev-r" style={{ transform: "rotate(180deg)" }} />
            Back
          </button>
        )}
        <h2>Changes</h2>
        <span className="sub">
          {files.length} file{files.length === 1 ? "" : "s"}
          {workspaceName ? ` · ${workspaceName}` : ""}
        </span>
        {totalPlus > 0 && <span className="plus">+{totalPlus}</span>}
        {totalMinus > 0 && <span className="minus">−{totalMinus}</span>}
        <span className="spacer" />
        {selectedFile && (
          <button
            type="button"
            className="btn sm"
            onClick={() => onOpenInEditor?.(selectedFile.path)}
          >
            Open in editor
          </button>
        )}
      </div>

      {files.length === 0 ? (
        <EmptyChanges workspaceRoot={workspaceRoot} />
      ) : (
        <div className="split">
          <div className="split-left">
            <div className="sec-label">Files written by Eury</div>
            {files.map((file) => (
              <div
                key={file.path}
                className={`row ${selectedFile?.path === file.path ? "on" : ""}`}
                onClick={() => setSelectedPath(file.path)}
              >
                <Icon
                  name={file.toolName === "delete_file" ? "x" : "pencil"}
                  style={{ color: "var(--color-warning)" }}
                />
                <span className="t">{file.name}</span>
                {file.plus > 0 && <span className="plus">+{file.plus}</span>}
                {file.minus > 0 && <span className="minus">−{file.minus}</span>}
              </div>
            ))}
          </div>

          <div className="split-right">
            <div className="inner" style={{ maxWidth: "none", padding: "14px" }}>
              {selectedFile && (
                <>
                  <div className="diff">
                    <div className="diff-head">
                      <span className="path">{selectedFile.path}</span>
                      {selectedFile.plus > 0 && <span className="plus">+{selectedFile.plus}</span>}
                      {selectedFile.minus > 0 && (
                        <span className="minus">−{selectedFile.minus}</span>
                      )}
                      <span className="status ok">
                        <Icon name="check" />
                        Applied
                      </span>
                      <span className="spacer" />
                      <div className="seg">
                        <span
                          className={diffMode === "unified" ? "on" : ""}
                          onClick={() => setDiffMode("unified")}
                        >
                          Unified
                        </span>
                        <span
                          className={diffMode === "split" ? "on" : ""}
                          onClick={() => setDiffMode("split")}
                        >
                          Split
                        </span>
                      </div>
                    </div>

                    <div className="diff-body">
                      {diffLines.length === 0 ? (
                        <div className="dl">
                          <span className="tx" style={{ color: "var(--color-fg-muted)" }}>
                            The {selectedFile.toolName} tool did not return a patch for this file.
                          </span>
                        </div>
                      ) : (
                        diffLines.map((line, idx) => (
                          <div
                            key={idx}
                            className={`dl ${
                              line.type === "add" ? "add" : line.type === "del" ? "del" : ""
                            }`}
                          >
                            <span className="ln">
                              {line.type === "del" ? line.oldLine : line.newLine ?? line.oldLine}
                            </span>
                            <span className="sg">
                              {line.type === "add" ? "+" : line.type === "del" ? "−" : " "}
                            </span>
                            <span className="tx">{line.content}</span>
                          </div>
                        ))
                      )}
                    </div>

                    <div className="diff-foot">
                      <span>{new Date(selectedFile.changedAt).toLocaleString()}</span>
                      <span className="spacer" />
                      <button
                        type="button"
                        className="btn sm ghost"
                        onClick={() => onOpenInEditor?.(selectedFile.path)}
                      >
                        <Icon name="file" />
                        Open in editor
                      </button>
                      {selectedFile.diff && (
                        <button
                          type="button"
                          className="btn sm ghost"
                          onClick={() => void navigator.clipboard.writeText(selectedFile.diff ?? "")}
                        >
                          <Icon name="copy" />
                          Copy diff
                        </button>
                      )}
                    </div>
                  </div>

                  <div className="sec-label">Why this changed</div>
                  <div className="setting">
                    <Icon name="activity" size={16} style={{ color: "var(--color-fg-subtle)" }} />
                    <div className="t">
                      Run <b>{selectedFile.runTitle}</b>
                      <div className="sub">{selectedFile.toolName} tool</div>
                    </div>
                    {onOpenRun && (
                      <button type="button" className="btn sm ghost" onClick={onOpenRun}>
                        Open run
                      </button>
                    )}
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
