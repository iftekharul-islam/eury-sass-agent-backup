import * as React from "react";
import "@xterm/xterm/css/xterm.css";
import { WebglAddon } from "@xterm/addon-webgl";
import { CanvasAddon } from "@xterm/addon-canvas";
import { getTerminal, getFitAddon, resizeSession } from "../../lib/terminal-session";

export interface TerminalViewProps {
  terminalId: string;
  /** Whether this tab is the currently visible one. The DOM node stays
   * mounted (never unmounted) so xterm's scrollback survives tab
   * switches — only visibility toggles. */
  active: boolean;
}

const RESIZE_DEBOUNCE_MS = 50;

export function TerminalView({ terminalId, active }: TerminalViewProps) {
  const containerRef = React.useRef<HTMLDivElement | null>(null);
  const mountedRef = React.useRef(false);
  const resizeTimerRef = React.useRef<number | undefined>(undefined);

  React.useEffect(() => {
    const container = containerRef.current;
    const term = getTerminal(terminalId);
    if (!container || !term || mountedRef.current) return;
    mountedRef.current = true;

    term.open(container);

    try {
      const webgl = new WebglAddon();
      webgl.onContextLoss(() => {
        webgl.dispose();
        term.loadAddon(new CanvasAddon());
      });
      term.loadAddon(webgl);
    } catch {
      term.loadAddon(new CanvasAddon());
    }

    const fit = getFitAddon(terminalId);
    fit?.fit();
  }, [terminalId]);

  React.useEffect(() => {
    if (!active) return;
    const fit = getFitAddon(terminalId);
    // A previously-hidden pane (display:none) has zero dimensions, so a fit
    // performed while hidden would compute garbage — refit on becoming visible.
    fit?.fit();
    const term = getTerminal(terminalId);
    if (fit && term) {
      void resizeSession(terminalId, term.cols, term.rows);
    }
  }, [active, terminalId]);

  React.useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const observer = new ResizeObserver(() => {
      if (!active) return;
      window.clearTimeout(resizeTimerRef.current);
      resizeTimerRef.current = window.setTimeout(() => {
        const fit = getFitAddon(terminalId);
        const term = getTerminal(terminalId);
        fit?.fit();
        if (term) {
          void resizeSession(terminalId, term.cols, term.rows);
        }
      }, RESIZE_DEBOUNCE_MS);
    });
    observer.observe(container);
    return () => {
      observer.disconnect();
      window.clearTimeout(resizeTimerRef.current);
    };
  }, [terminalId, active]);

  return (
    <div
      ref={containerRef}
      className="term-view"
      style={{ display: active ? "block" : "none" }}
    />
  );
}
