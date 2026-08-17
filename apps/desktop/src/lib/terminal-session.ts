import { Terminal, type IDisposable } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { Channel } from "@tauri-apps/api/core";
import { ipcClient } from "./ipc";
import { parseTerminalFrame } from "./terminal-events";
import { previewServers } from "./preview-servers";
import { registerTerminalLinks } from "./terminal-links";

/**
 * Module-level (non-React) registry of live xterm instances, one per
 * terminal session. Modeled on `events.ts`'s `useAgentEventStream`
 * rAF-batching *technique*, inverted: instead of batching into React
 * state, the channel callback batches PTY chunks and a single rAF flush
 * calls `term.write()` directly. React state (see `TerminalPane.tsx`) is
 * reserved for tab-level metadata only — title, status, dropped-byte
 * count — never the byte stream itself, which would be far too slow to
 * push through React on every chunk.
 */
export interface TerminalTabMeta {
  terminalId: string;
  title: string;
  cwd: string;
  status: "running" | "exited" | "killed";
  exitCode?: number;
  degraded?: string;
  droppedBytes: number;
}

type Listener = (meta: TerminalTabMeta) => void;

interface Entry {
  term: Terminal;
  fit: FitAddon;
  channel: Channel<unknown>;
  meta: TerminalTabMeta;
  listeners: Set<Listener>;
  pending: Uint8Array[];
  rafId: number;
  /**
   * Streaming decoder for the preview scanner. Stateful on purpose: a URL can
   * be split across two PTY chunks, and a fresh decoder per chunk would lose
   * the bytes straddling the boundary.
   */
  decoder: TextDecoder;
  /** Tail of the last decoded chunk, so a URL split across chunks still matches. */
  carry: string;
  /** The clickable-URL provider, disposed with the session. */
  links: IDisposable;
}

/** Enough to hold the longest URL a dev server is likely to print. */
const CARRY_CHARS = 256;

/** Feeds decoded output to the preview scanner without disturbing xterm. */
function scanForPreview(entry: Entry, chunk: Uint8Array) {
  const text = entry.decoder.decode(chunk, { stream: true });
  if (!text) return;
  const window = entry.carry + text;
  previewServers.observeOutput(entry.meta.terminalId, window);
  entry.carry = window.slice(-CARRY_CHARS);
}

const sessions = new Map<string, Entry>();

function resolveCssVar(name: string, fallback: string): string {
  if (typeof window === "undefined") return fallback;
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

function buildTheme() {
  return {
    background: resolveCssVar("--color-bg-inset", "#100f0f"),
    foreground: resolveCssVar("--color-fg", "#f0ede8"),
    cursor: resolveCssVar("--color-accent", "#e89a60"),
    selectionBackground: resolveCssVar("--color-bg-active", "#332f2c"),
  };
}

function flush(entry: Entry) {
  entry.rafId = 0;
  if (entry.pending.length === 0) return;
  const chunks = entry.pending;
  entry.pending = [];
  for (const chunk of chunks) {
    entry.term.write(chunk);
  }
}

function schedule(entry: Entry) {
  if (!entry.rafId) {
    entry.rafId = requestAnimationFrame(() => flush(entry));
  }
}

function notify(entry: Entry) {
  for (const listener of entry.listeners) listener(entry.meta);
}

export interface CreateSessionOptions {
  cwd?: string;
  shell?: string;
  cols: number;
  rows: number;
  /** Written into scrollback before the live prompt, for tool-card promotion (WI-8). */
  seedHistory?: string;
}

export async function createTerminalSession(opts: CreateSessionOptions): Promise<string> {
  const term = new Terminal({
    fontFamily: resolveCssVar("--font-mono", "monospace"),
    fontSize: 13,
    scrollback: 10_000,
    convertEol: true,
    theme: buildTheme(),
  });
  const fit = new FitAddon();
  term.loadAddon(fit);
  // A URL a dev server prints is the point of running it — make it clickable.
  const links = registerTerminalLinks(term);

  if (opts.seedHistory) {
    term.write(opts.seedHistory.replace(/\n/g, "\r\n"));
  }

  const channel = new Channel<unknown>();
  // Assigned synchronously right after `create` resolves, before any other
  // microtask can run — see the null guard below for why that matters.
  let entryRef: Entry | null = null;

  channel.onmessage = (raw: unknown) => {
    if (!entryRef) return;
    const entry = entryRef;
    const frame = parseTerminalFrame(raw);
    switch (frame.kind) {
      case "data":
        entry.pending.push(frame.bytes);
        scanForPreview(entry, frame.bytes);
        if (frame.droppedBefore > 0) {
          entry.meta.droppedBytes += frame.droppedBefore;
          notify(entry);
        }
        schedule(entry);
        break;
      case "exited":
        entry.meta.status = "exited";
        entry.meta.exitCode = frame.code;
        previewServers.markTerminalStopped(entry.meta.terminalId);
        notify(entry);
        break;
      case "killed":
        entry.meta.status = "killed";
        previewServers.markTerminalStopped(entry.meta.terminalId);
        notify(entry);
        break;
      case "degraded":
        entry.meta.degraded = frame.detail;
        notify(entry);
        break;
      case "unknown":
        break;
    }
  };

  const created = await ipcClient.terminal.create({
    cwd: opts.cwd,
    shell: opts.shell,
    cols: opts.cols,
    rows: opts.rows,
    onFrame: channel,
  });

  term.onData((data) => {
    void ipcClient.terminal.write(created.terminalId, data);
  });

  const entry: Entry = {
    term,
    fit,
    channel,
    meta: {
      terminalId: created.terminalId,
      title: opts.shell ?? "Terminal",
      cwd: opts.cwd ?? "",
      status: "running",
      degraded: created.degraded,
      droppedBytes: 0,
    },
    listeners: new Set(),
    pending: [],
    rafId: 0,
    decoder: new TextDecoder(),
    carry: "",
    links,
  };
  entryRef = entry;
  sessions.set(created.terminalId, entry);

  return created.terminalId;
}

export function getTerminal(id: string): Terminal | undefined {
  return sessions.get(id)?.term;
}

export function getFitAddon(id: string): FitAddon | undefined {
  return sessions.get(id)?.fit;
}

export function getMeta(id: string): TerminalTabMeta | undefined {
  return sessions.get(id)?.meta;
}

export function subscribeSession(id: string, listener: Listener): () => void {
  const entry = sessions.get(id);
  if (!entry) return () => {};
  entry.listeners.add(listener);
  return () => entry.listeners.delete(listener);
}

export async function resizeSession(id: string, cols: number, rows: number): Promise<void> {
  if (cols === 0 || rows === 0) return;
  await ipcClient.terminal.resize(id, cols, rows);
}

export async function closeSession(id: string): Promise<void> {
  previewServers.markTerminalStopped(id);
  const entry = sessions.get(id);
  if (entry) {
    if (entry.rafId) cancelAnimationFrame(entry.rafId);
    entry.links.dispose();
    entry.term.dispose();
    sessions.delete(id);
  }
  await ipcClient.terminal.close(id);
}

export async function captureSession(id: string, lines?: number): Promise<string> {
  const res = await ipcClient.terminal.capture(id, lines);
  return res.text;
}

/** Re-read CSS tokens after `data-theme` / accent changes on `<html>`. */
export function refreshTerminalThemes(): void {
  const theme = buildTheme();
  for (const entry of sessions.values()) {
    entry.term.options.theme = theme;
  }
}
