import * as React from "react";
import { Icon } from "./Icons";
import { TerminalTabStrip } from "./terminal/TerminalTabStrip";
import { TerminalView } from "./terminal/TerminalView";
import { MAX_TERMINAL_SESSIONS } from "../lib/terminal-constants";
import { ipcClient } from "../lib/ipc";
import {
  createTerminalSession,
  closeSession,
  captureSession,
  getMeta,
  getTerminal,
  subscribeSession,
  type TerminalTabMeta,
} from "../lib/terminal-session";

export interface TerminalPaneSeed {
  cwd?: string;
  /** Rendered as scrollback before the live prompt (tool-card promotion). */
  history?: string;
  /**
   * A command to execute on arrival — the Run button on a shell block in the
   * transcript. Reuses the live session when there is one, so running two
   * commands in a row does not pile up tabs.
   */
  command?: string;
}

export interface TerminalPaneProps {
  workspaceRoot?: string;
  tabs: string[];
  activeId: string | null;
  onTabsChange: (tabs: string[]) => void;
  onActiveChange: (id: string | null) => void;
  onShareOutput?: (text: string) => void;
  seed?: TerminalPaneSeed | null;
  onSeedConsumed?: () => void;
  /** Keeps the pane mounted but invisible so transcript Run can reach a live session. */
  offscreen?: boolean;
  onCommandError?: (message: string) => void;
}

export function TerminalPane({
  workspaceRoot,
  tabs,
  activeId,
  onTabsChange,
  onActiveChange,
  onShareOutput,
  seed,
  onSeedConsumed,
  offscreen = false,
  onCommandError,
}: TerminalPaneProps) {
  const [, forceUpdate] = React.useReducer((n: number) => n + 1, 0);
  const creatingRef = React.useRef(false);
  const seededRef = React.useRef<TerminalPaneSeed | null>(null);
  const [commandError, setCommandError] = React.useState<string | null>(null);

  const reportCommandError = React.useCallback(
    (message: string) => {
      console.error(message);
      setCommandError(message);
      onCommandError?.(message);
    },
    [onCommandError],
  );

  const create = React.useCallback(
    async (options?: TerminalPaneSeed): Promise<string | null> => {
      if (creatingRef.current || tabs.length >= MAX_TERMINAL_SESSIONS) return null;
      creatingRef.current = true;
      try {
        const id = await createTerminalSession({
          cwd: options?.cwd ?? workspaceRoot,
          cols: 80,
          rows: 24,
          seedHistory: options?.history,
        });
        onTabsChange([...tabs, id]);
        onActiveChange(id);
        return id;
      } catch (err) {
        const message =
          err instanceof Error ? err.message : "Failed to open terminal session";
        reportCommandError(message);
        return null;
      } finally {
        creatingRef.current = false;
      }
    },
    [tabs, workspaceRoot, onTabsChange, onActiveChange, reportCommandError],
  );

  /** Types a command into a session and presses Return. */
  const submit = React.useCallback(async (terminalId: string, command: string) => {
    try {
      await ipcClient.terminal.write(terminalId, `${command}\n`);
    } catch (err) {
      console.error("Failed to run command in terminal", err);
    }
  }, []);

  // Auto-open the first session when the pane mounts with none yet.
  React.useEffect(() => {
    if (tabs.length === 0 && !seed) {
      void create();
    }
    // Only on first mount with an empty tab list — opening more is user-driven.
  }, []);

  React.useEffect(() => {
    if (!seed?.command) return;
    setCommandError(null);
  }, [seed]);

  // Consume each seed exactly once — tracked by identity, not by a one-shot
  // flag, so a second Run while this pane is already open still lands.
  React.useEffect(() => {
    if (!seed || seededRef.current === seed) return;
    seededRef.current = seed;

    void (async () => {
      const { command } = seed;
      const live = tabs.find((id) => getMeta(id)?.status === "running");

      try {
        if (command && live) {
          onActiveChange(live);
          await submit(live, command);
        } else {
          const id = await create(seed);
          if (!id && command) {
            reportCommandError(
              "Could not open a terminal session. Trust this project if you have not already.",
            );
            return;
          }
          // A freshly spawned shell has not printed its prompt yet; typing into
          // it immediately can land before the line discipline is ready.
          if (id && command) {
            window.setTimeout(() => void submit(id, command), 250);
          }
        }
      } finally {
        seededRef.current = null;
        onSeedConsumed?.();
      }
    })();
  }, [seed, tabs, create, submit, onActiveChange, onSeedConsumed, reportCommandError]);

  // Re-render whenever any open session's metadata changes (title, status,
  // dropped-byte count) — the byte stream itself never touches this state.
  React.useEffect(() => {
    const unsubs = tabs.map((id) => subscribeSession(id, () => forceUpdate()));
    return () => unsubs.forEach((u) => u());
  }, [tabs]);

  const metas: TerminalTabMeta[] = tabs
    .map((id) => getMeta(id))
    .filter((m): m is TerminalTabMeta => Boolean(m));

  const handleClose = React.useCallback(
    (id: string) => {
      void closeSession(id);
      const remaining = tabs.filter((t) => t !== id);
      onTabsChange(remaining);
      if (activeId === id) {
        onActiveChange(remaining[remaining.length - 1] ?? null);
      }
    },
    [tabs, activeId, onTabsChange, onActiveChange],
  );

  const handleClear = () => {
    if (activeId) getTerminal(activeId)?.clear();
  };

  const handleScrollBottom = () => {
    if (activeId) getTerminal(activeId)?.scrollToBottom();
  };

  const handleShare = async () => {
    if (!activeId || !onShareOutput) return;
    const text = await captureSession(activeId, 200);
    onShareOutput(text);
  };

  const activeMeta = metas.find((m) => m.terminalId === activeId);

  return (
    <div className={`term-pane${offscreen ? " is-offscreen" : ""}`}>
      <div className="term-toolbar">
        <TerminalTabStrip
          tabs={metas}
          activeId={activeId}
          onSelect={onActiveChange}
          onClose={handleClose}
          onCreate={() => void create()}
        />
        <div className="term-actions">
          <button type="button" className="tb-icon" title="Clear" onClick={handleClear}>
            <Icon name="slash" size={16} />
          </button>
          <button type="button" className="tb-icon" title="Scroll to bottom" onClick={handleScrollBottom}>
            <Icon name="chev-d" size={16} />
          </button>
          <button
            type="button"
            className="tb-icon"
            title="Kill this session"
            onClick={() => activeId && handleClose(activeId)}
          >
            <Icon name="stop" size={16} />
          </button>
          {onShareOutput && (
            <button type="button" className="term-share-btn" title="Share output with agent" onClick={() => void handleShare()}>
              <Icon name="send" size={14} />
              Share output
            </button>
          )}
        </div>
      </div>

      {activeMeta?.degraded && (
        <div className="term-degraded-banner">
          <Icon name="alert" size={14} />
          {activeMeta.degraded}
        </div>
      )}

      {commandError && (
        <div className="term-degraded-banner">
          <Icon name="alert" size={14} />
          {commandError}
        </div>
      )}

      <div className="term-body">
        {tabs.length === 0 ? (
          <div className="term-empty">
            <Icon name="terminal" size={20} />
            <span>No terminal sessions open</span>
          </div>
        ) : (
          tabs.map((id) => <TerminalView key={id} terminalId={id} active={id === activeId} />)
        )}
      </div>
    </div>
  );
}
