import { Channel, invoke } from "@tauri-apps/api/core";

export interface WorkspaceInfo {
  path: string;
  name: string;
  is_trusted: boolean;
}

/// The settings blob as it crosses the IPC boundary. The Rust side reasons
/// about `theme`/`accent`/`density` and round-trips everything else opaquely,
/// so this deliberately does not restate the full UI settings schema — that
/// lives in `lib/settings.ts`, which owns it.
export type SettingsBlob = {
  theme: string;
  accent: string;
  density: string;
} & Record<string, unknown>;

export interface HistoryMessage {
  role: "user" | "assistant";
  content: string;
}

export interface RunRequest {
  runId: string;
  conversationId: string;
  mode: "chat" | "ask" | "plan" | "agent" | "build";
  prompt: string;
  /** Prior turns of this conversation, oldest first, so follow-ups have context. */
  history?: HistoryMessage[];
  attachments?: unknown[];
  workspaceId?: string;
  workspaceRoot?: string;
  model: {
    provider: string;
    model: string;
    temperature: number;
    maxTokens?: number;
  };
  planContext?: unknown;
}

export interface TerminalInfo {
  terminalId: string;
  workspaceId: string;
  title: string;
  cwd: string;
  shell: string;
  cols: number;
  rows: number;
  pid?: number;
  status: { state: "running" } | { state: "exited"; code?: number } | { state: "killed" };
  createdAt: number;
  degraded?: string;
}

export interface TerminalCreated {
  terminalId: string;
  degraded?: string;
}

/** One node of the workspace file tree, path relative to the workspace root. */
export interface WorkspaceEntry {
  path: string;
  name: string;
  isDir: boolean;
  depth: number;
}

export interface WorkspaceFile {
  path: string;
  content: string;
  truncated: boolean;
}

export interface Capabilities {
  ipcApiVersion: number;
  eventSpecVersion: string;
  appVersion: string;
  buildSha: string;
  channel: string;
  platform: { os: string; arch: string; osVersion: string };
  features: Record<string, boolean>;
  sandbox: { available: boolean; kind: "seatbelt" | "landlock" | "job_object" | "none" };
  limits: { maxFileSizeMb: number; maxTerminals: number; maxParallelTools: number };
  offline: boolean;
  airGapped: boolean;
}

export const ipcClient = {
  workspace: {
    pickFolder: () => invoke<string | null>("workspace_pick_folder"),
    open: (path?: string) => invoke<WorkspaceInfo>("workspace_open", { path }),
    close: () => invoke<void>("workspace_close"),
    info: () => invoke<WorkspaceInfo>("workspace_info"),
    recent: () => invoke<WorkspaceInfo[]>("workspace_recent"),
    setTrust: (path: string, trust: boolean) => invoke<void>("workspace_trust_set", { path, trust }),
    // Read-class, so these work in an untrusted workspace: the user can browse
    // a project before deciding to trust it.
    listTree: (subPath?: string, maxDepth?: number) =>
      invoke<WorkspaceEntry[]>("workspace_list_tree", { subPath, maxDepth }),
    readFile: (path: string) => invoke<WorkspaceFile>("workspace_read_file", { path }),
  },
  settings: {
    get: () => invoke<SettingsBlob>("settings_get"),
    set: (settings: SettingsBlob) => invoke<void>("settings_set", { settings }),
  },
  capabilities: {
    get: () => invoke<Capabilities>("capabilities_get"),
  },
  window: {
    saveState: () => invoke<void>("window_state_save"),
    loadState: () => invoke<void>("window_state_load"),
  },
  run: {
    start: (request: RunRequest) => invoke<void>("run_start", { request }),
    cancel: (conversationId: string) => invoke<void>("run_cancel", { conversationId }),
    approve: (toolCallId: string, decision: 'allow_once' | 'allow_session' | 'denied') =>
      invoke<void>('run_approve', {
        toolCallId,
        approved: decision !== 'denied',
        scope: decision,
      }),
    steer: (conversationId: string, instruction: string) => invoke<void>("run_steer", { conversationId, instruction }),
  },
  terminal: {
    create: (args: {
      cwd?: string;
      shell?: string;
      cols: number;
      rows: number;
      onFrame: Channel<unknown>;
    }) => invoke<TerminalCreated>("terminal_create", args),
    write: (terminalId: string, data: string) => invoke<void>("terminal_write", { terminalId, data }),
    resize: (terminalId: string, cols: number, rows: number) =>
      invoke<void>("terminal_resize", { terminalId, cols, rows }),
    close: (terminalId: string) => invoke<void>("terminal_close", { terminalId }),
    list: () => invoke<TerminalInfo[]>("terminal_list"),
    capture: (terminalId: string, lines?: number) =>
      invoke<{ text: string }>("terminal_capture", { terminalId, lines }),
  },
};

