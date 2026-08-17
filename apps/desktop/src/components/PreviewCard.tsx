import * as React from "react";
import { Icon } from "./Icons";
import { openExternalUrl } from "../lib/open";
import { previewServers, type PreviewServer } from "../lib/preview-servers";

/** Width the framed page is laid out at, then scaled down to fit the card. */
const PREVIEW_WIDTH = 1280;
const PREVIEW_HEIGHT = 800;

export interface PreviewCardProps {
  server: PreviewServer;
}

/**
 * The running app, in the conversation.
 *
 * Starting a dev server used to end at a line of output the user had to carry
 * into a browser themselves. This shows the page instead: a live frame of it,
 * where it is served, and one button to open it properly.
 *
 * The frame is loopback-only — the CSP grants `frame-src` to localhost and
 * nothing else, so this can never render a remote page.
 */
export function PreviewCard({ server }: PreviewCardProps) {
  const [scale, setScale] = React.useState(0.4);
  const [reloadKey, setReloadKey] = React.useState(0);
  const [menuOpen, setMenuOpen] = React.useState(false);
  const [copied, setCopied] = React.useState(false);
  const [openError, setOpenError] = React.useState<string | null>(null);
  const frameRef = React.useRef<HTMLDivElement>(null);
  const menuRef = React.useRef<HTMLDivElement>(null);
  const stopped = server.status === "stopped";

  // The page renders at desktop width and is scaled to the column, so the
  // preview shows the layout the user will actually get.
  React.useEffect(() => {
    const el = frameRef.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    const observer = new ResizeObserver(([entry]) => {
      const width = entry.contentRect.width;
      if (width > 0) setScale(width / PREVIEW_WIDTH);
    });
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  React.useEffect(() => {
    if (!menuOpen) return;
    const onPointerDown = (event: PointerEvent) => {
      if (!menuRef.current?.contains(event.target as Node)) setMenuOpen(false);
    };
    window.addEventListener("pointerdown", onPointerDown);
    return () => window.removeEventListener("pointerdown", onPointerDown);
  }, [menuOpen]);

  const open = async () => {
    setOpenError(null);
    try {
      await openExternalUrl(server.url);
    } catch (err) {
      setOpenError(err instanceof Error ? err.message : "Could not open the browser.");
    }
  };

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(server.url);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      /* clipboard unavailable */
    }
    setMenuOpen(false);
  };

  return (
    <div className={`preview-card${stopped ? " is-stopped" : ""}`}>
      <div className="preview-chrome">
        <span className="preview-dots" aria-hidden="true">
          <i />
          <i />
          <i />
        </span>
        <span className="preview-chrome-url">{`${server.host}:${server.port}`}</span>
        <span className="spacer" />
        <span className={`preview-state${stopped ? " is-stopped" : ""}`}>
          {stopped ? "Stopped" : "Live"}
        </span>
      </div>

      <div className="preview-viewport" ref={frameRef}>
        {stopped ? (
          <div className="preview-stopped-note">
            <Icon name="stop" size={16} />
            <span>This server is no longer running.</span>
          </div>
        ) : (
          <iframe
            key={reloadKey}
            className="preview-frame"
            src={server.url}
            title={`Preview of ${server.host}:${server.port}`}
            sandbox="allow-scripts allow-forms allow-popups allow-same-origin"
            style={{
              width: PREVIEW_WIDTH,
              height: PREVIEW_HEIGHT,
              transform: `scale(${scale})`,
            }}
          />
        )}
      </div>

      <div className="preview-foot">
        <div className="preview-id">
          <strong>{`${server.host}:${server.port}`}</strong>
          <span>{stopped ? "Stopped" : "Running from the terminal"}</span>
        </div>
        <span className="spacer" />
        <button type="button" className="btn sm" onClick={() => void open()} disabled={stopped}>
          Open
        </button>
        <div className="preview-menu" ref={menuRef}>
          <button
            type="button"
            className="tb-icon"
            aria-label="Preview options"
            aria-expanded={menuOpen}
            onClick={() => setMenuOpen((value) => !value)}
          >
            <Icon name="more" size={15} />
          </button>
          {menuOpen && (
            <div className="preview-menu-pop" role="menu">
              <button type="button" className="row" onClick={() => void copy()}>
                {copied ? "Copied" : "Copy URL"}
              </button>
              <button
                type="button"
                className="row"
                disabled={stopped}
                onClick={() => {
                  setReloadKey((key) => key + 1);
                  setMenuOpen(false);
                }}
              >
                Reload
              </button>
              <button
                type="button"
                className="row"
                onClick={() => {
                  previewServers.dismiss(server.id);
                  setMenuOpen(false);
                }}
              >
                Dismiss
              </button>
            </div>
          )}
        </div>
      </div>

      {openError && <div className="preview-error">{openError}</div>}
    </div>
  );
}
