/**
 * Live session state for the Code area.
 *
 * Everything the conversation, changes, runs, approvals and context surfaces
 * render comes from here, and everything in here comes from a real run: turns
 * the user actually sent, tool calls the core actually emitted, approvals the
 * user actually decided. There is no seeded content — an install that has
 * never run the agent has an empty store, and the views say so.
 */
import { useSyncExternalStore } from "react";
import { stripAnsi } from "./ansi";
import { stripToolCallFences } from "./assistant-text";
import { parseStreamEvent } from "./stream-events";

export type RunStatus = "running" | "completed" | "failed" | "cancelled";
export type ApprovalDecision = "allow_once" | "allow_session" | "denied";

export type ActivityStatus =
  | "running"
  | "succeeded"
  | "failed"
  | "pending_approval"
  | "denied";

export interface ToolActivityRecord {
  id: string;
  runId: string;
  conversationId: string;
  name: string;
  payload?: Record<string, unknown>;
  status: ActivityStatus;
  startedAt: number;
  endedAt?: number;
  durationMs?: number;
  stdout?: string;
  stderr?: string;
  diff?: string;
  /** Set for write-class tools once the path is known. */
  path?: string;
  plus?: number;
  minus?: number;
  exitCode?: number;
}

export interface ApprovalRecord {
  /** Tool call id — the handle `run_approve` expects. */
  id: string;
  runId: string;
  conversationId: string;
  workspaceRoot?: string;
  toolName: string;
  target: string;
  requestedAt: number;
  decidedAt?: number;
  decision?: ApprovalDecision;
}

export type TurnBlock =
  | { kind: "text"; text: string }
  | { kind: "tool"; toolCallId: string };

export interface TranscriptTurn {
  id: string;
  runId?: string;
  sender: "user" | "assistant";
  who: string;
  avatar: string;
  at: number;
  badge?: string;
  modeBadge?: string;
  fileChip?: string;
  /** Reasoning text, when the model streams any. */
  thinking?: string;
  /** Set while reasoning is streaming; cleared when the model moves on. */
  thinkingStartedAt?: number;
  /** Total time spent reasoning, for the "Thought for 8s" label. */
  thinkingMs?: number;
  blocks: TurnBlock[];
}

export interface RunUsage {
  inputTokens?: number;
  outputTokens?: number;
  costUsd?: number;
}

/** Context pressure, as last reported by the core for a run. */
export interface RunContext {
  tokens: number;
  limit: number;
  message?: string;
}

/** What the run is doing right now, for the live status line. */
export type RunPhase = "starting" | "thinking" | "responding" | "tool" | "awaiting_approval";

export interface RunRecord {
  id: string;
  conversationId: string;
  workspaceRoot?: string;
  workspaceName?: string;
  title: string;
  mode: string;
  modelLabel: string;
  startedAt: number;
  endedAt?: number;
  status: RunStatus;
  error?: string;
  usage?: RunUsage;
  context?: RunContext;
  activityIds: string[];
  phase: RunPhase;
  /** When the core last said anything — drives the "no response yet" state. */
  lastEventAt: number;
  /** Characters of assistant text streamed so far, for the live counter. */
  streamedChars: number;
}

export interface SessionState {
  /** Transcript turns keyed by conversation id. */
  transcripts: Record<string, TranscriptTurn[]>;
  /** Run history, oldest first. */
  runs: RunRecord[];
  /** Every tool call seen, keyed by tool call id. */
  activities: Record<string, ToolActivityRecord>;
  /** Approval requests and decisions, oldest first. */
  approvals: ApprovalRecord[];
}

const STORAGE_KEY = "eury_session_state_v1";
const MAX_RUNS = 50;
const MAX_TURNS_PER_CONVERSATION = 400;
const MAX_APPROVALS = 200;
const PERSIST_DEBOUNCE_MS = 500;

const EMPTY_STATE: SessionState = {
  transcripts: {},
  runs: [],
  activities: {},
  approvals: [],
};

function loadPersisted(): SessionState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return EMPTY_STATE;
    const parsed = JSON.parse(raw) as Partial<SessionState>;
    return {
      transcripts: parsed.transcripts ?? {},
      runs: parsed.runs ?? [],
      activities: parsed.activities ?? {},
      approvals: parsed.approvals ?? [],
    };
  } catch {
    return EMPTY_STATE;
  }
}

let state: SessionState = loadPersisted();

// A run that was still `running` when the app was closed never resumes, so it
// is reconciled to `cancelled` rather than spinning forever in the UI.
if (state.runs.some((run) => run.status === "running")) {
  state = {
    ...state,
    // A turn left mid-reasoning would otherwise render as forever "Thinking…".
    transcripts: Object.fromEntries(
      Object.entries(state.transcripts).map(([id, turns]) => [
        id,
        turns.map((turn) =>
          turn.thinkingStartedAt
            ? { ...turn, thinkingStartedAt: undefined, thinkingMs: turn.thinkingMs ?? 0 }
            : turn,
        ),
      ]),
    ),
    runs: state.runs.map((run) =>
      run.status === "running"
        ? { ...run, status: "cancelled" as RunStatus, endedAt: run.endedAt ?? Date.now() }
        : run,
    ),
    activities: Object.fromEntries(
      Object.entries(state.activities).map(([id, act]) => [
        id,
        act.status === "running" ? { ...act, status: "failed" as ActivityStatus } : act,
      ]),
    ),
  };
}

const listeners = new Set<() => void>();
let persistTimer: ReturnType<typeof setTimeout> | null = null;

function schedulePersist() {
  if (persistTimer) clearTimeout(persistTimer);
  persistTimer = setTimeout(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch {
      // Quota or unavailable storage: the in-memory session still works.
    }
  }, PERSIST_DEBOUNCE_MS);
}

function commit(next: SessionState) {
  state = next;
  schedulePersist();
  listeners.forEach((listener) => listener());
}

function trimTurns(turns: TranscriptTurn[]): TranscriptTurn[] {
  return turns.length > MAX_TURNS_PER_CONVERSATION
    ? turns.slice(turns.length - MAX_TURNS_PER_CONVERSATION)
    : turns;
}

function trimRuns(runs: RunRecord[]): RunRecord[] {
  return runs.length > MAX_RUNS ? runs.slice(runs.length - MAX_RUNS) : runs;
}

/** Which run new events belong to. Set on `startRun`, cleared when it ends. */
let activeRunId: string | null = null;

function runById(id: string | null): RunRecord | undefined {
  if (!id) return undefined;
  return state.runs.find((run) => run.id === id);
}

function targetRunId(rawRunId?: string): string | null {
  if (rawRunId && state.runs.some((run) => run.id === rawRunId)) return rawRunId;
  return activeRunId;
}

function toRecord(value: unknown): Record<string, unknown> | undefined {
  return value && typeof value === "object" ? (value as Record<string, unknown>) : undefined;
}

function firstString(source: Record<string, unknown> | undefined, keys: string[]): string | undefined {
  if (!source) return undefined;
  for (const key of keys) {
    const value = source[key];
    if (typeof value === "string" && value.length > 0) return value;
  }
  return undefined;
}

function firstNumber(source: Record<string, unknown> | undefined, keys: string[]): number | undefined {
  if (!source) return undefined;
  for (const key of keys) {
    const value = source[key];
    if (typeof value === "number" && Number.isFinite(value)) return value;
  }
  return undefined;
}

/** Counts added/removed lines in a unified diff, ignoring the file headers. */
export function countDiffLines(diff: string): { plus: number; minus: number } {
  let plus = 0;
  let minus = 0;
  for (const line of diff.split("\n")) {
    if (line.startsWith("+++") || line.startsWith("---")) continue;
    if (line.startsWith("+")) plus += 1;
    else if (line.startsWith("-")) minus += 1;
  }
  return { plus, minus };
}

const WRITE_TOOLS = new Set(["write_file", "edit_file", "apply_patch", "create_file", "delete_file"]);

export function isWriteTool(name: string): boolean {
  return WRITE_TOOLS.has(name);
}

export function activityTarget(activity: {
  payload?: Record<string, unknown>;
  path?: string;
  name: string;
}): string {
  return (
    firstString(activity.payload, ["command", "path", "file_path", "pattern", "query", "url"]) ??
    activity.path ??
    activity.name
  );
}

function titleFromPrompt(prompt: string): string {
  const trimmed = prompt.trim().replace(/\s+/g, " ");
  if (!trimmed) return "Untitled run";
  return trimmed.length > 64 ? `${trimmed.slice(0, 64)}…` : trimmed;
}

export interface StartRunInput {
  runId: string;
  conversationId: string;
  prompt: string;
  mode: string;
  modelLabel: string;
  workspaceRoot?: string;
  workspaceName?: string;
}

export interface AppendUserTurnInput {
  conversationId: string;
  who: string;
  avatar: string;
  text: string;
  fileChip?: string;
  modeBadge?: string;
}

export const sessionStore = {
  getState(): SessionState {
    return state;
  },

  subscribe(listener: () => void): () => void {
    listeners.add(listener);
    return () => listeners.delete(listener);
  },

  appendUserTurn(input: AppendUserTurnInput): TranscriptTurn {
    const turn: TranscriptTurn = {
      id: `u-${Date.now()}-${Math.round(performance.now())}`,
      sender: "user",
      who: input.who,
      avatar: input.avatar,
      at: Date.now(),
      modeBadge: input.modeBadge,
      fileChip: input.fileChip,
      blocks: input.text ? [{ kind: "text", text: input.text }] : [],
    };
    const existing = state.transcripts[input.conversationId] ?? [];
    commit({
      ...state,
      transcripts: {
        ...state.transcripts,
        [input.conversationId]: trimTurns([...existing, turn]),
      },
    });
    return turn;
  },

  startRun(input: StartRunInput): RunRecord {
    const run: RunRecord = {
      id: input.runId,
      conversationId: input.conversationId,
      workspaceRoot: input.workspaceRoot,
      workspaceName: input.workspaceName,
      title: titleFromPrompt(input.prompt),
      mode: input.mode,
      modelLabel: input.modelLabel,
      startedAt: Date.now(),
      status: "running",
      activityIds: [],
      phase: "thinking",
      lastEventAt: Date.now(),
      streamedChars: 0,
    };
    activeRunId = run.id;

    // The assistant turn opens empty; text deltas and tool cards fill it in.
    const assistantTurn: TranscriptTurn = {
      id: `a-${run.id}`,
      runId: run.id,
      sender: "assistant",
      who: "Eury",
      avatar: "E",
      at: Date.now(),
      badge: input.modelLabel,
      modeBadge: input.mode,
      blocks: [],
      thinkingStartedAt: Date.now(),
    };
    const existing = state.transcripts[input.conversationId] ?? [];

    commit({
      ...state,
      runs: trimRuns([...state.runs, run]),
      transcripts: {
        ...state.transcripts,
        [input.conversationId]: trimTurns([...existing, assistantTurn]),
      },
    });
    return run;
  },

  failRun(runId: string, message: string) {
    const run = runById(runId);
    // A run the user already stopped stays stopped: a late failure from the
    // core must not reopen or relabel a finished run.
    if (!run || run.status !== "running") return;
    if (activeRunId === runId) activeRunId = null;
    const base = sealThinkingForRun(state, run);
    commit({
      ...base,
      runs: base.runs.map((r) =>
        r.id === runId
          ? { ...r, status: "failed" as RunStatus, endedAt: Date.now(), error: message }
          : r,
      ),
    });
  },

  cancelActiveRun(conversationId: string) {
    const run = [...state.runs]
      .reverse()
      .find((r) => r.conversationId === conversationId && r.status === "running");
    if (!run) return;
    if (activeRunId === run.id) activeRunId = null;
    const base = sealThinkingForRun(state, run);
    commit({
      ...base,
      runs: base.runs.map((r) =>
        r.id === run.id ? { ...r, status: "cancelled" as RunStatus, endedAt: Date.now() } : r,
      ),
      activities: Object.fromEntries(
        Object.entries(base.activities).map(([id, act]) => [
          id,
          act.runId === run.id && (act.status === "running" || act.status === "pending_approval")
            ? { ...act, status: "failed" as ActivityStatus, endedAt: Date.now() }
            : act,
        ]),
      ),
    });
  },

  recordApprovalDecision(toolCallId: string, decision: ApprovalDecision) {
    const activity = state.activities[toolCallId];
    const nextStatus: ActivityStatus = decision === "denied" ? "denied" : "running";
    commit({
      ...state,
      approvals: state.approvals.map((record) =>
        record.id === toolCallId && !record.decidedAt
          ? { ...record, decision, decidedAt: Date.now() }
          : record,
      ),
      activities: activity
        ? {
            ...state.activities,
            [toolCallId]: {
              ...activity,
              status: nextStatus,
              startedAt: decision === "denied" ? activity.startedAt : Date.now(),
              endedAt: decision === "denied" ? Date.now() : undefined,
            },
          }
        : state.activities,
    });
  },

  clearConversation(conversationId: string) {
    if (!state.transcripts[conversationId]) return;
    const nextTranscripts = { ...state.transcripts };
    delete nextTranscripts[conversationId];
    commit({ ...state, transcripts: nextTranscripts });
  },

  /** Test seam: drops everything, including what was persisted. */
  reset() {
    activeRunId = null;
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch {
      // ignore
    }
    commit({ transcripts: {}, runs: [], activities: {}, approvals: [] });
  },

  ingestEvent(raw: unknown) {
    const parsed = parseStreamEvent(raw);
    const rawRecord = toRecord(raw);
    const payload = toRecord(rawRecord?.payload) ?? rawRecord;
    const runId = targetRunId(firstString(payload, ["run_id", "runId"]));

    // Meta is not transcript content, but it proves the core is alive.
    if (parsed.kind === "meta") {
      if (runId) markProgress(runId, "meta");
      return;
    }

    if (parsed.kind === "other") return;

    const run = runById(runId);
    if (!run || !runId) return;

    // Every event is a sign of life: the status line reads this to tell
    // "still working" apart from "nothing has come back".
    markProgress(runId, parsed.kind);

    switch (parsed.kind) {
      case "text_delta":
        appendAssistantText(run, parsed.text);
        break;
      case "thinking_delta":
        appendAssistantThinking(run, parsed.text);
        break;
      case "tool_start":
        addActivity(run, {
          id: parsed.toolCallId,
          name: parsed.name,
          payload: toRecord(parsed.arguments),
          status: "running",
        });
        break;
      case "approval_required":
        addActivity(run, {
          id: parsed.toolCallId,
          name: parsed.name,
          payload: toRecord(parsed.arguments),
          status: "pending_approval",
        });
        break;
      case "tool_end":
        finishActivity(parsed.toolCallId, toRecord(parsed.result));
        break;
      case "tool_output_delta":
        appendToolOutput(parsed.toolCallId, parsed.stream, parsed.text);
        break;
      case "cost_update":
        updateRun(runId, {
          usage: {
            inputTokens: parsed.tokensPrompt,
            outputTokens: parsed.tokensCompletion,
            costUsd: parsed.costUsdMicros / 1_000_000,
          },
        });
        break;
      case "context_warning":
        updateRun(runId, {
          context: { tokens: parsed.tokens, limit: parsed.limit, message: parsed.message },
        });
        break;
      case "run_complete":
        completeRun(runId, payload);
        break;
      case "run_error":
        this.failRun(runId, parsed.message ?? "Run failed");
        // A run rejected for auth means the session is gone, not that this
        // one request was unlucky — send the user back to sign-in.
        if (parsed.code === "EURY_AUTH_UNAUTHORIZED") {
          void import("./auth").then((m) => m.handleUnauthorized());
        }
        break;
    }
  },
};

const PHASE_BY_EVENT: Record<string, RunPhase> = {
  text_delta: "responding",
  thinking_delta: "thinking",
  tool_start: "tool",
  tool_output_delta: "tool",
  tool_end: "thinking",
  approval_required: "awaiting_approval",
};

/** Records that the core is alive, and what it is doing. */
function markProgress(runId: string, kind: string) {
  const run = runById(runId);
  if (!run || run.status !== "running") return;
  const phase = PHASE_BY_EVENT[kind] ?? run.phase;
  if (run.phase === phase && Date.now() - run.lastEventAt < 250) return;
  commit({
    ...state,
    runs: state.runs.map((r) =>
      r.id === runId ? { ...r, phase, lastEventAt: Date.now() } : r,
    ),
  });
}

function withAssistantTurn(
  run: RunRecord,
  update: (turn: TranscriptTurn) => TranscriptTurn,
): SessionState {
  const turns = state.transcripts[run.conversationId] ?? [];
  const index = turns.findIndex((turn) => turn.runId === run.id && turn.sender === "assistant");
  if (index < 0) return state;
  const next = [...turns];
  next[index] = update(next[index]);
  return {
    ...state,
    transcripts: { ...state.transcripts, [run.conversationId]: next },
  };
}

/**
 * Closes the reasoning window on a turn. Reasoning is over the moment the model
 * starts answering, calls a tool, or the run ends — that is what "Thought for
 * 8s" measures, not the whole run.
 */
function sealThinking(turn: TranscriptTurn): TranscriptTurn {
  if (!turn.thinkingStartedAt) return turn;
  return {
    ...turn,
    thinkingMs: (turn.thinkingMs ?? 0) + (Date.now() - turn.thinkingStartedAt),
    thinkingStartedAt: undefined,
  };
}

/** `sealThinking` for a run that is ending, applied to a whole state value. */
function sealThinkingForRun(s: SessionState, run: RunRecord): SessionState {
  const turns = s.transcripts[run.conversationId];
  if (!turns) return s;
  const index = turns.findIndex((turn) => turn.runId === run.id && turn.sender === "assistant");
  if (index < 0 || !turns[index].thinkingStartedAt) return s;
  const next = [...turns];
  next[index] = sealThinking(next[index]);
  return { ...s, transcripts: { ...s.transcripts, [run.conversationId]: next } };
}

function appendAssistantText(run: RunRecord, text: string) {
  if (!text) return;
  const withCount = withAssistantTurn(run, (turn) => {
    turn = sealThinking(turn);
    const blocks = [...turn.blocks];
    const last = blocks[blocks.length - 1];
    if (last?.kind === "text") {
      blocks[blocks.length - 1] = { kind: "text", text: last.text + text };
    } else {
      blocks.push({ kind: "text", text });
    }
    return { ...turn, blocks };
  });

  commit({
    ...withCount,
    runs: withCount.runs.map((r) =>
      r.id === run.id ? { ...r, streamedChars: r.streamedChars + text.length } : r,
    ),
  });
}

function appendAssistantThinking(run: RunRecord, text: string) {
  if (!text) return;
  const next = withAssistantTurn(run, (turn) => ({
    ...turn,
    thinking: (turn.thinking ?? "") + text,
    // A second burst of reasoning after an answer opens a new window; the
    // durations add up rather than counting the answering time in between.
    thinkingStartedAt: turn.thinkingStartedAt ?? Date.now(),
  }));
  commit({
    ...next,
    runs: next.runs.map((r) =>
      r.id === run.id ? { ...r, streamedChars: r.streamedChars + text.length } : r,
    ),
  });
}

function addActivity(
  run: RunRecord,
  input: {
    id: string;
    name: string;
    payload?: Record<string, unknown>;
    status: ActivityStatus;
  },
) {
  if (!input.id) return;

  const existing = state.activities[input.id];
  if (existing?.runId === run.id) {
    // `tool_start` after the user already approved in the UI — refresh payload
    // and keep the transcript block that `approval_required` opened.
    if (input.status === "running" && existing.status !== "succeeded" && existing.status !== "failed") {
      commit({
        ...state,
        activities: {
          ...state.activities,
          [input.id]: {
            ...existing,
            name: input.name,
            payload: input.payload ?? existing.payload,
            status: "running",
            startedAt: Date.now(),
            endedAt: undefined,
          },
        },
      });
    }
    return;
  }

  // Prior runs reused ids like `call_0_0`; drop stale records so a new run's
  // tool events are not silently swallowed.
  const activities = { ...state.activities };
  if (existing) {
    delete activities[input.id];
  }

  const activity: ToolActivityRecord = {
    id: input.id,
    runId: run.id,
    conversationId: run.conversationId,
    name: input.name,
    payload: input.payload,
    status: input.status,
    startedAt: Date.now(),
    path: firstString(input.payload, ["path", "file_path"]),
  };

  const nextApprovals =
    input.status === "pending_approval"
      ? [
          ...state.approvals,
          {
            id: input.id,
            runId: run.id,
            conversationId: run.conversationId,
            workspaceRoot: run.workspaceRoot,
            toolName: input.name,
            target: activityTarget(activity),
            requestedAt: Date.now(),
          } satisfies ApprovalRecord,
        ].slice(-MAX_APPROVALS)
      : state.approvals;

  const withBlock = withAssistantTurn(run, (turn) => ({
    ...sealThinking(turn),
    blocks: [...turn.blocks, { kind: "tool" as const, toolCallId: input.id }],
  }));

  const runs = withBlock.runs.map((r) => {
    if (r.id === run.id) {
      return { ...r, activityIds: [...r.activityIds.filter((id) => id !== input.id), input.id] };
    }
    return { ...r, activityIds: r.activityIds.filter((id) => id !== input.id) };
  });

  commit({
    ...withBlock,
    activities: { ...activities, [input.id]: activity },
    runs,
    approvals: nextApprovals,
  });
}

function appendToolOutput(toolCallId: string, stream: "stdout" | "stderr", text: string) {
  const activity = state.activities[toolCallId];
  if (!activity) return;
  if (activeRunId && activity.runId !== activeRunId) return;
  if (!text) return;

  const chunk = stripAnsi(text);
  const key = stream === "stderr" ? "stderr" : "stdout";

  commit({
    ...state,
    activities: {
      ...state.activities,
      [toolCallId]: {
        ...activity,
        [key]: `${activity[key] ?? ""}${chunk}`,
      },
    },
  });
}

function finishActivity(toolCallId: string, result?: Record<string, unknown>) {
  const activity = state.activities[toolCallId];
  if (!activity) return;
  if (activeRunId && activity.runId !== activeRunId) return;

  const stdout = firstString(result, ["stdout", "output", "content"]);
  const stderr = firstString(result, ["stderr", "error"]);
  const diff = firstString(result, ["diff", "patch"]);
  const exitCode = firstNumber(result, ["exit_code", "exitCode", "status"]);
  const counted = diff ? countDiffLines(diff) : undefined;
  const endedAt = Date.now();
  const failed =
    (typeof exitCode === "number" && exitCode !== 0) ||
    result?.ok === false ||
    (!!(stderr ?? activity.stderr) && !(stdout ?? activity.stdout) && !diff);

  commit({
    ...state,
    activities: {
      ...state.activities,
      [toolCallId]: {
        ...activity,
        status: failed ? "failed" : "succeeded",
        endedAt,
        durationMs: endedAt - activity.startedAt,
        stdout: stdout ? stripAnsi(stdout) : activity.stdout,
        stderr: stderr ? stripAnsi(stderr) : activity.stderr,
        diff,
        exitCode,
        path: activity.path ?? firstString(result, ["path", "file_path"]),
        plus: firstNumber(result, ["added", "plus", "lines_added"]) ?? counted?.plus,
        minus: firstNumber(result, ["removed", "minus", "lines_removed"]) ?? counted?.minus,
      },
    },
  });
}

function updateRun(runId: string, patch: Partial<RunRecord>) {
  commit({
    ...state,
    runs: state.runs.map((run) => (run.id === runId ? { ...run, ...patch } : run)),
  });
}

function completeRun(runId: string, payload?: Record<string, unknown>) {
  const run = runById(runId);
  if (!run || run.status !== "running") return;
  if (activeRunId === runId) activeRunId = null;
  const usageRecord = toRecord(payload?.usage) ?? payload;
  const usage: RunUsage = {
    inputTokens: firstNumber(usageRecord, ["input_tokens", "inputTokens", "prompt_tokens"]),
    outputTokens: firstNumber(usageRecord, ["output_tokens", "outputTokens", "completion_tokens"]),
    costUsd: firstNumber(usageRecord, ["cost_usd", "costUsd"]),
  };
  const hasUsage = Object.values(usage).some((value) => typeof value === "number");
  const endedAt = Date.now();
  const base = sealThinkingForRun(state, run);

  commit({
    ...base,
    activities: Object.fromEntries(
      Object.entries(base.activities).map(([id, act]) => [
        id,
        act.runId === runId && act.status === "running"
          ? { ...act, status: "failed" as ActivityStatus, endedAt }
          : act,
      ]),
    ),
    runs: base.runs.map((run) =>
      run.id === runId
        ? {
            ...run,
            status: "completed" as RunStatus,
            endedAt,
            ...(hasUsage ? { usage } : {}),
          }
        : run,
    ),
  });
}

/* ------------------------------------------------------------------ */
/* Event bridge                                                        */
/* ------------------------------------------------------------------ */

let bridgeStarted = false;

/**
 * Pipes the core's event topic into the store. Safe to call more than once;
 * outside Tauri (unit tests, plain browser) it resolves to a no-op teardown.
 */
export async function startSessionEventBridge(topic = "agent://app"): Promise<() => void> {
  if (bridgeStarted) return () => {};
  bridgeStarted = true;
  try {
    const { listen } = await import("@tauri-apps/api/event");
    const unlisten = await listen<unknown>(topic, (event) => {
      sessionStore.ingestEvent(event.payload);
    });
    return () => {
      bridgeStarted = false;
      unlisten();
    };
  } catch {
    bridgeStarted = false;
    return () => {};
  }
}

/* ------------------------------------------------------------------ */
/* Selectors / hooks                                                   */
/* ------------------------------------------------------------------ */

export function useSessionState(): SessionState {
  return useSyncExternalStore(sessionStore.subscribe, sessionStore.getState, sessionStore.getState);
}

export function selectTranscript(s: SessionState, conversationId: string): TranscriptTurn[] {
  return s.transcripts[conversationId] ?? [];
}

export interface HistoryMessage {
  role: "user" | "assistant";
  content: string;
}

/**
 * The conversation so far, flattened for the model. Tool cards are left out —
 * the core replays tool results itself — and empty turns are dropped so a
 * still-streaming assistant turn doesn't become a blank message.
 */
export function selectHistory(
  s: SessionState,
  conversationId: string,
  maxTurns = 20,
): HistoryMessage[] {
  const turns = s.transcripts[conversationId] ?? [];
  const messages: HistoryMessage[] = [];

  for (const turn of turns) {
    const raw = turn.blocks
      .filter((block): block is { kind: "text"; text: string } => block.kind === "text")
      .map((block) => block.text)
      .join("");
    // Replaying a tool-call fence would read as a call the model still owes an
    // answer to; the core already ran it and fed back the result.
    const content = turn.sender === "assistant" ? stripToolCallFences(raw) : raw.trim();
    if (!content) continue;
    messages.push({ role: turn.sender, content });
  }

  return messages.slice(-maxTurns);
}

export function selectActiveRun(s: SessionState, conversationId: string): RunRecord | undefined {
  return [...s.runs]
    .reverse()
    .find((run) => run.conversationId === conversationId && run.status === "running");
}

export function selectLatestRun(s: SessionState, conversationId: string): RunRecord | undefined {
  return [...s.runs].reverse().find((run) => run.conversationId === conversationId);
}

export function selectRunActivities(s: SessionState, runId: string): ToolActivityRecord[] {
  const run = s.runs.find((r) => r.id === runId);
  if (!run) return [];
  return run.activityIds.map((id) => s.activities[id]).filter(Boolean);
}

export function selectRuns(s: SessionState, workspaceRoot?: string): RunRecord[] {
  const runs = workspaceRoot
    ? s.runs.filter((run) => run.workspaceRoot === workspaceRoot)
    : s.runs;
  return [...runs].reverse();
}

export function selectPendingApprovals(s: SessionState, workspaceRoot?: string): ApprovalRecord[] {
  return s.approvals.filter(
    (record) =>
      !record.decidedAt && (!workspaceRoot || record.workspaceRoot === workspaceRoot),
  );
}

export function selectPendingApprovalsForRun(
  s: SessionState,
  runId: string,
): ApprovalRecord[] {
  return s.approvals.filter((record) => record.runId === runId && !record.decidedAt);
}

export function selectDecidedApprovals(s: SessionState, workspaceRoot?: string): ApprovalRecord[] {
  return [...s.approvals]
    .filter(
      (record) => !!record.decidedAt && (!workspaceRoot || record.workspaceRoot === workspaceRoot),
    )
    .reverse();
}

export interface ChangedFile {
  path: string;
  name: string;
  plus: number;
  minus: number;
  diff?: string;
  toolName: string;
  activityId: string;
  runId: string;
  runTitle: string;
  changedAt: number;
}

/**
 * Files this app actually wrote, newest write per path, derived from
 * completed write-class tool calls.
 */
export function selectChangedFiles(s: SessionState, workspaceRoot?: string): ChangedFile[] {
  const byPath = new Map<string, ChangedFile>();

  for (const run of s.runs) {
    if (workspaceRoot && run.workspaceRoot !== workspaceRoot) continue;
    for (const id of run.activityIds) {
      const activity = s.activities[id];
      if (!activity || activity.status !== "succeeded") continue;
      if (!isWriteTool(activity.name)) continue;
      const path = activity.path;
      if (!path) continue;
      byPath.set(path, {
        path,
        name: path.split("/").filter(Boolean).pop() ?? path,
        plus: activity.plus ?? 0,
        minus: activity.minus ?? 0,
        diff: activity.diff,
        toolName: activity.name,
        activityId: activity.id,
        runId: run.id,
        runTitle: run.title,
        changedAt: activity.endedAt ?? activity.startedAt,
      });
    }
  }

  return [...byPath.values()].sort((a, b) => b.changedAt - a.changedAt);
}
