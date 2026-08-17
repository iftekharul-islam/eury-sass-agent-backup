import * as React from "react";
import { EuryMark } from "./EuryMark";
import { Icon } from "./Icons";
import {
  Platform,
  windowClose,
  windowMinimize,
  windowToggleMaximize,
  windowToggleFullscreen,
  windowIsMaximized,
  windowStartDragging,
} from "../lib/window";

export interface TitleBarProps {
  area: "home" | "code";
  onAreaChange: (area: "home" | "code") => void;
  workspaceName?: string;
  branchName?: string;
  isDirty?: boolean;
  mode?: "Chat" | "Agent" | "Plan" | "Ask" | "Build";
  onModeChange?: (mode: "Chat" | "Agent" | "Plan" | "Ask" | "Build") => void;
  onOpenCommands?: () => void;
  onOpenChanges?: () => void;
  onOpenBrowser?: () => void;
  onOpenFiles?: () => void;
  onOpenSearch?: () => void;
  isAuthenticated?: boolean;
}

export function TitleBar({
  area,
  onAreaChange,
  workspaceName,
  branchName,
  isDirty = false,
  mode = "Agent",
  onModeChange,
  onOpenCommands,
  onOpenChanges,
  onOpenBrowser,
  onOpenFiles,
  onOpenSearch,
  isAuthenticated = true,
}: TitleBarProps) {
  const [showModeMenu, setShowModeMenu] = React.useState(false);
  const [isMac, setIsMac] = React.useState<boolean>(() => Platform.isMac());
  const [isMaximized, setIsMaximized] = React.useState(false);

  const modes: Array<"Chat" | "Agent" | "Plan" | "Ask" | "Build"> = [
    "Chat",
    "Agent",
    "Plan",
    "Ask",
    "Build",
  ];

  // Refresh platform state and maximized state on mount
  React.useEffect(() => {
    setIsMac(Platform.isMac());
    windowIsMaximized().then(setIsMaximized).catch(() => {});
  }, []);

  const handleDoubleClick = async (e: React.MouseEvent) => {
    const target = e.target as HTMLElement;
    if (target.closest("button, a, input, select, textarea, .tb-icon, .tb-btn, .area-tab, .traffic, .win-controls, [data-tauri-drag-region='false']")) {
      return;
    }
    await windowToggleMaximize();
    const max = await windowIsMaximized();
    setIsMaximized(max);
  };

  const handleMouseDown = (e: React.MouseEvent) => {
    // Only trigger drag on primary mouse button when clicking the non-interactive header background
    if (e.button === 0) {
      const target = e.target as HTMLElement;
      if (!target.closest("button, a, input, select, textarea, .tb-icon, .tb-btn, .area-tab, .traffic, .win-controls, [data-tauri-drag-region='false']")) {
        windowStartDragging();
      }
    }
  };

  const handleClose = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    await windowClose();
  };

  const handleMinimize = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    await windowMinimize();
  };

  const handleMaximize = async (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.altKey) {
      await windowToggleFullscreen();
    } else {
      await windowToggleMaximize();
    }
    const max = await windowIsMaximized();
    setIsMaximized(max);
  };

  const trafficLights = isMac ? (
    <span className="traffic" aria-label="Window controls" data-tauri-drag-region="false">
      <button
        type="button"
        className="close"
        onClick={handleClose}
        title="Close window (⌘W)"
        aria-label="Close window"
      >
        <svg viewBox="0 0 10 10">
          <path d="M2.5 2.5L7.5 7.5M7.5 2.5L2.5 7.5" />
        </svg>
      </button>
      <button
        type="button"
        className="minimize"
        onClick={handleMinimize}
        title="Minimize window (⌘M)"
        aria-label="Minimize window"
      >
        <svg viewBox="0 0 10 10">
          <path d="M2 5h6" />
        </svg>
      </button>
      <button
        type="button"
        className="maximize"
        onClick={handleMaximize}
        title="Maximize window (Option-click for Fullscreen)"
        aria-label="Maximize window"
      >
        <svg viewBox="0 0 10 10">
          <path d="M2.5 5h5M5 2.5v5" />
        </svg>
      </button>
    </span>
  ) : null;

  return (
    <header
      className={`titlebar${isMac ? " is-mac" : ""}`}
      data-tauri-drag-region
      onDoubleClick={handleDoubleClick}
      onMouseDown={handleMouseDown}
    >
      {!isAuthenticated ? (
        <>
          <div className="titlebar-leading" data-tauri-drag-region="false">
            {trafficLights}
            <div className="titlebar-brand">
              <EuryMark size={18} />
              <span className="titlebar-brand-name">Eury Agent</span>
              <span className="titlebar-badge">Sign in</span>
            </div>
          </div>
          <span className="spacer" data-tauri-drag-region />
        </>
      ) : (
        <>
          <div className="titlebar-leading" data-tauri-drag-region="false">
            {trafficLights}
            <nav className="area-tabs" aria-label="Product area">
              <button
                type="button"
                className={`area-tab ${area === "home" ? "on" : ""}`}
                data-area="home"
                aria-current={area === "home" ? "page" : "false"}
                onClick={(e) => {
                  e.stopPropagation();
                  onAreaChange("home");
                }}
              >
                <Icon name="home" />
                Home
              </button>
              <button
                type="button"
                className={`area-tab ${area === "code" ? "on" : ""}`}
                data-area="code"
                aria-current={area === "code" ? "page" : "false"}
                onClick={(e) => {
                  e.stopPropagation();
                  onAreaChange("code");
                }}
              >
                <Icon name="code" />
                Code
              </button>
            </nav>
          </div>

          {/* Code breadcrumbs — always mounted; hidden on Home via CSS to avoid titlebar jump */}
          <div className="titlebar-context project-context" data-tauri-drag-region="false">
            <button
              type="button"
              className="tb-btn project-context"
              title="Switch workspace"
              onClick={(e) => {
                e.stopPropagation();
                if (onOpenFiles) onOpenFiles();
              }}
            >
              <Icon name="folder" />
              <span id="tb-ws">{workspaceName ?? "Open project"}</span>
              <Icon name="chev-d" />
            </button>

            {/* The branch chip only appears once a branch is actually known. */}
            {branchName && (
              <div className="tb-branch project-context" title="Active Git branch">
                <Icon name="branch" />
                <span>{branchName}</span>
                {isDirty && <i className="dirty" title="Uncommitted changes" />}
              </div>
            )}
          </div>

          <span className="spacer" data-tauri-drag-region />

          {/* Workspace Tools — always mounted; hidden outside Code via CSS */}
          <div className="workspace-tools" aria-label="Workspace tools" data-tauri-drag-region="false">
            <button
              type="button"
              className="tb-icon"
              data-nav="palette"
              title="Commands (⌘K)"
              aria-label="Open commands"
              onClick={(e) => {
                e.stopPropagation();
                if (onOpenCommands) onOpenCommands();
              }}
            >
              <Icon name="terminal" size={16} />
            </button>
            <button
              type="button"
              className="tb-icon"
              data-nav="changes"
              title="Changes (⌘⇧G)"
              aria-label="Open changes"
              onClick={(e) => {
                e.stopPropagation();
                if (onOpenChanges) onOpenChanges();
              }}
            >
              <Icon name="diff" size={16} />
            </button>
            <button
              type="button"
              className="tb-icon"
              data-nav="browser"
              title="Browser preview"
              aria-label="Open browser preview"
              onClick={(e) => {
                e.stopPropagation();
                if (onOpenBrowser) onOpenBrowser();
              }}
            >
              <Icon name="globe" size={16} />
            </button>
            <button
              type="button"
              className="tb-icon"
              data-nav="files"
              title="Files"
              aria-label="Open files"
              onClick={(e) => {
                e.stopPropagation();
                if (onOpenFiles) onOpenFiles();
              }}
            >
              <Icon name="folder" size={16} />
            </button>
          </div>

          <div className="titlebar-trailing" data-tauri-drag-region="false">
            <div style={{ position: "relative" }}>
              <button
                type="button"
                className="tb-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  setShowModeMenu((prev) => !prev);
                }}
                aria-expanded={showModeMenu}
                aria-haspopup="true"
              >
                <span id="tb-mode">{mode}</span>
                <Icon name="chev-d" />
              </button>

              {showModeMenu && (
                <div
                  style={{
                    position: "absolute",
                    top: "100%",
                    right: 0,
                    marginTop: "4px",
                    background: "var(--color-bg-elevated)",
                    border: "1px solid var(--color-border-strong)",
                    borderRadius: "var(--r-md)",
                    boxShadow: "var(--sh-popover)",
                    zIndex: 100,
                    minWidth: "120px",
                    padding: "4px",
                  }}
                >
                  {modes.map((m) => (
                    <button
                      key={m}
                      type="button"
                      className={`row ${m === mode ? "on" : ""}`}
                      style={{ padding: "4px 8px", fontSize: "12px" }}
                      onClick={(e) => {
                        e.stopPropagation();
                        if (onModeChange) onModeChange(m);
                        setShowModeMenu(false);
                      }}
                    >
                      <span className="t">{m}</span>
                    </button>
                  ))}
                </div>
              )}
            </div>

            <button
              type="button"
              className="tb-icon"
              title="Search (⌘P)"
              aria-label="Search"
              onClick={(e) => {
                e.stopPropagation();
                if (onOpenSearch) onOpenSearch();
                else if (onOpenCommands) onOpenCommands();
              }}
            >
              <Icon name="search" size={16} />
            </button>
          </div>
        </>
      )}

      {/* Windows / Linux Titlebar Controls (Top Right) */}
      {!isMac && (
        <div
          className="win-controls"
          aria-label="Window controls"
          data-tauri-drag-region="false"
        >
          <button
            type="button"
            className="win-btn"
            title="Minimize"
            aria-label="Minimize window"
            onClick={handleMinimize}
            data-tauri-drag-region="false"
          >
            <svg viewBox="0 0 10 10">
              <path d="M1 5h8" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
            </svg>
          </button>
          <button
            type="button"
            className="win-btn"
            title={isMaximized ? "Restore" : "Maximize"}
            aria-label={isMaximized ? "Restore window" : "Maximize window"}
            onClick={handleMaximize}
            data-tauri-drag-region="false"
          >
            {isMaximized ? (
              <svg viewBox="0 0 10 10">
                <path d="M2.5 1.5h5a1 1 0 0 1 1 1v5M1.5 3.5h5a1 1 0 0 1 1 1v5H1.5z" stroke="currentColor" strokeWidth="1" fill="none" />
              </svg>
            ) : (
              <svg viewBox="0 0 10 10">
                <rect x="1.5" y="1.5" width="7" height="7" rx="1" stroke="currentColor" strokeWidth="1" fill="none" />
              </svg>
            )}
          </button>
          <button
            type="button"
            className="win-btn close"
            title="Close"
            aria-label="Close window"
            onClick={handleClose}
            data-tauri-drag-region="false"
          >
            <svg viewBox="0 0 10 10">
              <path d="M2 2l6 6M8 2L2 8" stroke="currentColor" strokeWidth="1.1" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      )}
    </header>
  );
}
