/**
 * Local servers this session started.
 *
 * A dev server announces itself in its own output ("Local: http://localhost:5173/")
 * and nowhere else — no tool result carries it, because the process never
 * exits to return one. So the terminal stream is the source: every chunk is
 * scanned for a loopback URL, and what it finds becomes a live preview in the
 * conversation instead of a line the user has to copy into a browser.
 *
 * In-memory by design. The servers are children of this session's PTYs, so a
 * restart leaves nothing running to preview.
 */
import { useSyncExternalStore } from "react";
import { stripAnsi } from "./ansi";

export type PreviewStatus = "live" | "stopped";

export interface PreviewServer {
  /** Normalized origin — the identity of the server. */
  id: string;
  /** Origin plus the first path it advertised, which is what gets opened. */
  url: string;
  host: string;
  port: string;
  /** The terminal session it was announced in; drives the stopped state. */
  terminalId: string;
  detectedAt: number;
  lastSeenAt: number;
  status: PreviewStatus;
}

export interface PreviewState {
  servers: PreviewServer[];
}

const MAX_SERVERS = 8;

let state: PreviewState = { servers: [] };
const listeners = new Set<() => void>();

function commit(next: PreviewState) {
  state = next;
  listeners.forEach((listener) => listener());
}

/**
 * Escape sequences, which is what colours a dev server's banner — the URL
 * inside one is wrapped in them, so they come off before matching.
 */
const LOOPBACK_URL =
  /https?:\/\/(localhost|127\.0\.0\.1|0\.0\.0\.0|\[::1\]):(\d{2,5})(\/[^\s"'`<>]*)?/gi;

/** `0.0.0.0` is an address to bind, not one to browse. */
function normalizeHost(host: string): string {
  const lower = host.toLowerCase();
  return lower === "0.0.0.0" || lower === "[::1]" ? "localhost" : lower;
}

export interface DetectedUrl {
  id: string;
  url: string;
  host: string;
  port: string;
}

/** Every loopback URL in a chunk of terminal output, newest-first order kept. */
export function detectPreviewUrls(text: string): DetectedUrl[] {
  const clean = stripAnsi(text);
  const found = new Map<string, DetectedUrl>();

  for (const match of clean.matchAll(LOOPBACK_URL)) {
    const host = normalizeHost(match[1]);
    const port = match[2];
    // A trailing dot or bracket is sentence punctuation, not part of the path.
    const path = (match[3] ?? "/").replace(/[.,;:)\]}'"]+$/, "");
    const id = `http://${host}:${port}`;
    if (!found.has(id)) {
      found.set(id, { id, url: `${id}${path || "/"}`, host, port });
    }
  }

  return [...found.values()];
}

export const previewServers = {
  getState(): PreviewState {
    return state;
  },

  subscribe(listener: () => void): () => void {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },

  /** Scans a chunk of terminal output and records anything serving. */
  observeOutput(terminalId: string, text: string) {
    const detected = detectPreviewUrls(text);
    if (detected.length === 0) return;

    const now = Date.now();
    let servers = state.servers;
    let changed = false;

    for (const hit of detected) {
      const existing = servers.find((server) => server.id === hit.id);
      if (existing) {
        // A server that reprints its banner is alive again — a restarted dev
        // server keeps the same card rather than piling up a new one.
        if (existing.status === "stopped" || existing.terminalId !== terminalId) {
          servers = servers.map((server) =>
            server.id === hit.id
              ? { ...server, status: "live" as PreviewStatus, terminalId, lastSeenAt: now }
              : server,
          );
          changed = true;
        }
        continue;
      }

      servers = [
        ...servers,
        {
          id: hit.id,
          url: hit.url,
          host: hit.host,
          port: hit.port,
          terminalId,
          detectedAt: now,
          lastSeenAt: now,
          status: "live" as PreviewStatus,
        },
      ].slice(-MAX_SERVERS);
      changed = true;
    }

    if (changed) commit({ servers });
  },

  /** The session hosting these servers ended, so they are no longer serving. */
  markTerminalStopped(terminalId: string) {
    if (!state.servers.some((s) => s.terminalId === terminalId && s.status === "live")) return;
    commit({
      servers: state.servers.map((server) =>
        server.terminalId === terminalId
          ? { ...server, status: "stopped" as PreviewStatus }
          : server,
      ),
    });
  },

  dismiss(id: string) {
    if (!state.servers.some((server) => server.id === id)) return;
    commit({ servers: state.servers.filter((server) => server.id !== id) });
  },

  /** Test seam. */
  reset() {
    commit({ servers: [] });
  },
};

export function usePreviewServers(): PreviewState {
  return useSyncExternalStore(
    previewServers.subscribe,
    previewServers.getState,
    previewServers.getState,
  );
}

/** The server worth showing: the most recent one still serving, if any. */
export function selectActivePreview(s: PreviewState): PreviewServer | undefined {
  const live = s.servers.filter((server) => server.status === "live");
  const pool = live.length > 0 ? live : s.servers;
  return [...pool].sort((a, b) => b.lastSeenAt - a.lastSeenAt)[0];
}
