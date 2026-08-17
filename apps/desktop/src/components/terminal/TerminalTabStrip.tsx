import * as React from "react";
import { Icon } from "../Icons";
import { MAX_TERMINAL_SESSIONS } from "../../lib/terminal-constants";
import type { TerminalTabMeta } from "../../lib/terminal-session";

export interface TerminalTabStripProps {
  tabs: TerminalTabMeta[];
  activeId: string | null;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
  onCreate: () => void;
}

export function TerminalTabStrip({ tabs, activeId, onSelect, onClose, onCreate }: TerminalTabStripProps) {
  const atCap = tabs.length >= MAX_TERMINAL_SESSIONS;

  return (
    <nav className="term-tabs" aria-label="Terminal sessions">
      {tabs.map((tab) => (
        <button
          key={tab.terminalId}
          type="button"
          className={`term-tab ${tab.terminalId === activeId ? "on" : ""}`}
          aria-current={tab.terminalId === activeId}
          onClick={() => onSelect(tab.terminalId)}
        >
          <span className={`term-tab-dot term-tab-dot-${tab.status}`} aria-hidden="true" />
          <span className="term-tab-title">{tab.title}</span>
          {tab.droppedBytes > 0 && (
            <span className="term-tab-dropped" title={`${tab.droppedBytes} bytes dropped from the live view`}>
              !
            </span>
          )}
          <span
            className="term-tab-close"
            role="button"
            aria-label={`Close ${tab.title}`}
            onClick={(e) => {
              e.stopPropagation();
              onClose(tab.terminalId);
            }}
          >
            <Icon name="x" size={12} />
          </span>
        </button>
      ))}
      <button
        type="button"
        className="term-tab-new"
        disabled={atCap}
        title={atCap ? `Terminal limit reached (${MAX_TERMINAL_SESSIONS} per workspace)` : "New terminal"}
        onClick={onCreate}
      >
        <Icon name="plus" />
      </button>
    </nav>
  );
}
