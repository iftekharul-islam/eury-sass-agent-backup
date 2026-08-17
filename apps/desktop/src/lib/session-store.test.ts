import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  countDiffLines,
  selectActiveRun,
  selectChangedFiles,
  selectHistory,
  selectPendingApprovals,
  selectRunActivities,
  selectTranscript,
  sessionStore,
} from "./session-store";

const CONVERSATION = "conv-1";
const RUN = "run-1";

function startRun() {
  sessionStore.appendUserTurn({
    conversationId: CONVERSATION,
    who: "You",
    avatar: "U",
    text: "fix the failing test",
  });
  sessionStore.startRun({
    runId: RUN,
    conversationId: CONVERSATION,
    prompt: "fix the failing test",
    mode: "Agent",
    modelLabel: "Opus 5",
    workspaceRoot: "/tmp/project",
    workspaceName: "project",
  });
}

beforeEach(() => {
  vi.stubGlobal("localStorage", {
    getItem: () => null,
    setItem: () => {},
    removeItem: () => {},
  });
  sessionStore.reset();
});

describe("session store", () => {
  it("starts with nothing to show", () => {
    const state = sessionStore.getState();
    expect(selectTranscript(state, CONVERSATION)).toEqual([]);
    expect(state.runs).toEqual([]);
    expect(selectChangedFiles(state)).toEqual([]);
    expect(selectPendingApprovals(state)).toEqual([]);
  });

  it("records the user turn and an assistant turn for the run", () => {
    startRun();
    const turns = selectTranscript(sessionStore.getState(), CONVERSATION);
    expect(turns).toHaveLength(2);
    expect(turns[0].sender).toBe("user");
    expect(turns[1].sender).toBe("assistant");
    expect(selectActiveRun(sessionStore.getState(), CONVERSATION)?.id).toBe(RUN);
  });

  it("appends streamed text into the assistant turn", () => {
    startRun();
    sessionStore.ingestEvent({ type: "text_delta", payload: { text: "I'll " } });
    sessionStore.ingestEvent({ type: "text_delta", payload: { text: "reproduce it." } });

    const assistant = selectTranscript(sessionStore.getState(), CONVERSATION)[1];
    expect(assistant.blocks).toEqual([{ kind: "text", text: "I'll reproduce it." }]);
  });

  it("interleaves tool cards with text in the order they arrived", () => {
    startRun();
    sessionStore.ingestEvent({ type: "text_delta", payload: { text: "Running tests." } });
    sessionStore.ingestEvent({
      type: "tool_start",
      payload: { tool_call_id: "t1", name: "run_command", arguments: { command: "pnpm test" } },
    });
    sessionStore.ingestEvent({
      type: "tool_end",
      payload: { tool_call_id: "t1", result: { stdout: "1 failed", exit_code: 1 } },
    });

    const assistant = selectTranscript(sessionStore.getState(), CONVERSATION)[1];
    expect(assistant.blocks.map((b) => b.kind)).toEqual(["text", "tool"]);

    const activity = selectRunActivities(sessionStore.getState(), RUN)[0];
    expect(activity.status).toBe("failed");
    expect(activity.stdout).toBe("1 failed");
  });

  it("appends live command output while a tool is running", () => {
    startRun();
    sessionStore.ingestEvent({
      type: "tool_start",
      payload: { tool_call_id: "t-live", name: "run_command", arguments: { command: "npm install" } },
    });
    sessionStore.ingestEvent({
      type: "tool_output_delta",
      payload: { tool_call_id: "t-live", stream: "stdout", text: "added 10 packages\n" },
    });
    sessionStore.ingestEvent({
      type: "tool_output_delta",
      payload: { tool_call_id: "t-live", stream: "stderr", text: "npm warn deprecated\n" },
    });

    const activity = sessionStore.getState().activities["t-live"];
    expect(activity.status).toBe("running");
    expect(activity.stdout).toBe("added 10 packages\n");
    expect(activity.stderr).toBe("npm warn deprecated\n");
  });

  it("tracks approvals from request to decision", () => {
    startRun();
    sessionStore.ingestEvent({
      type: "approval_required",
      payload: { tool_call_id: "t2", name: "run_command", arguments: { command: "rm -rf .tmp" } },
    });

    expect(selectPendingApprovals(sessionStore.getState(), "/tmp/project")).toHaveLength(1);

    sessionStore.recordApprovalDecision("t2", "denied");
    const state = sessionStore.getState();
    expect(selectPendingApprovals(state, "/tmp/project")).toHaveLength(0);
    expect(state.approvals[0].decision).toBe("denied");
    expect(state.activities.t2.status).toBe("denied");
  });

  it("replaces stale tool-call ids from a prior run", () => {
    startRun();
    sessionStore.ingestEvent({
      type: "tool_start",
      payload: { tool_call_id: "call_0_0", name: "write_file", arguments: { path: "old.txt" } },
    });
    sessionStore.ingestEvent({
      type: "tool_end",
      payload: { tool_call_id: "call_0_0", result: { diff: "+old" } },
    });
    sessionStore.ingestEvent({ type: "run_complete", payload: { stop_reason: "stop" } });

    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "try again",
    });
    sessionStore.startRun({
      runId: "run-2",
      conversationId: CONVERSATION,
      prompt: "try again",
      mode: "Agent",
      modelLabel: "Opus 5",
      workspaceRoot: "/tmp/project",
      workspaceName: "project",
    });
    sessionStore.ingestEvent({
      type: "approval_required",
      payload: {
        tool_call_id: "call_0_0",
        name: "write_file",
        arguments: { path: "new.txt", content: "hello" },
      },
    });

    const state = sessionStore.getState();
    expect(state.activities.call_0_0.runId).toBe("run-2");
    expect(state.activities.call_0_0.payload?.path).toBe("new.txt");
    expect(selectPendingApprovals(state, "/tmp/project")).toHaveLength(1);
  });

  it("derives changed files from successful write tools", () => {
    startRun();
    sessionStore.ingestEvent({
      type: "tool_start",
      payload: {
        tool_call_id: "t3",
        name: "edit_file",
        arguments: { path: "src/auth/session.test.ts" },
      },
    });
    sessionStore.ingestEvent({
      type: "tool_end",
      payload: {
        tool_call_id: "t3",
        result: { diff: "@@ -1,2 +1,3 @@\n ctx\n-old\n+new\n+extra" },
      },
    });

    const [file] = selectChangedFiles(sessionStore.getState(), "/tmp/project");
    expect(file.path).toBe("src/auth/session.test.ts");
    expect(file.plus).toBe(2);
    expect(file.minus).toBe(1);
    // A different workspace sees none of it.
    expect(selectChangedFiles(sessionStore.getState(), "/other")).toEqual([]);
  });

  it("closes the run out on completion", () => {
    startRun();
    sessionStore.ingestEvent({
      type: "run_complete",
      payload: { stop_reason: "stop", usage: { input_tokens: 100, output_tokens: 20 } },
    });

    const state = sessionStore.getState();
    expect(selectActiveRun(state, CONVERSATION)).toBeUndefined();
    expect(state.runs[0].status).toBe("completed");
    expect(state.runs[0].usage?.inputTokens).toBe(100);
  });

  it("records cost and context pressure the core reports mid-run", () => {
    startRun();
    sessionStore.ingestEvent({
      type: "cost_update",
      payload: { tokens_prompt: 8200, tokens_completion: 1100, cost_usd_micros: 40_000 },
    });
    sessionStore.ingestEvent({
      type: "context_warning",
      payload: { tokens: 190_000, limit: 200_000, message: "Approaching context quota" },
    });

    const run = sessionStore.getState().runs[0];
    expect(run.usage).toEqual({ inputTokens: 8200, outputTokens: 1100, costUsd: 0.04 });
    expect(run.context).toEqual({
      tokens: 190_000,
      limit: 200_000,
      message: "Approaching context quota",
    });

    // Completing the run must not wipe the usage already reported.
    sessionStore.ingestEvent({ type: "run_complete", payload: { stop_reason: "stop" } });
    expect(sessionStore.getState().runs[0].usage?.costUsd).toBe(0.04);
  });

  it("marks the run failed on a run_error", () => {
    startRun();
    sessionStore.ingestEvent({
      type: "run_error",
      payload: { code: "EURY_PROVIDER", message: "provider unavailable" },
    });

    const state = sessionStore.getState();
    expect(state.runs[0].status).toBe("failed");
    expect(state.runs[0].error).toBe("provider unavailable");
  });

  it("flattens the transcript into history for the next run", () => {
    startRun();
    sessionStore.ingestEvent({ type: "text_delta", payload: { text: "On it." } });
    sessionStore.ingestEvent({
      type: "tool_start",
      payload: { tool_call_id: "t9", name: "list_dir", arguments: { path: "." } },
    });

    const history = selectHistory(sessionStore.getState(), CONVERSATION);
    // Tool cards stay out of history — the core replays tool results itself.
    expect(history).toEqual([
      { role: "user", content: "fix the failing test" },
      { role: "assistant", content: "On it." },
    ]);
  });

  it("keeps tool-call fences out of the replayed history", () => {
    startRun();
    sessionStore.ingestEvent({
      type: "text_delta",
      payload: {
        text: 'Looking.\n\n```tool_call\n{"name":"list_dir","arguments":{"path":"."}}\n```',
      },
    });

    const history = selectHistory(sessionStore.getState(), CONVERSATION);
    expect(history[1]).toEqual({ role: "assistant", content: "Looking." });
  });

  it("leaves an empty assistant turn out of history", () => {
    startRun();
    expect(selectHistory(sessionStore.getState(), CONVERSATION)).toEqual([
      { role: "user", content: "fix the failing test" },
    ]);
  });

  it("keeps a stopped run stopped when the core reports late", () => {
    startRun();
    sessionStore.cancelActiveRun(CONVERSATION);
    expect(sessionStore.getState().runs[0].status).toBe("cancelled");

    // The core is still winding down and reports afterwards; neither event
    // may reopen or relabel a run the user already stopped.
    sessionStore.ingestEvent({ type: "run_complete", payload: { stop_reason: "stop" } });
    sessionStore.ingestEvent({ type: "run_error", payload: { message: "too late" } });

    const run = sessionStore.getState().runs[0];
    expect(run.status).toBe("cancelled");
    expect(run.error).toBeUndefined();
  });

  it("ignores events that arrive with no run in flight", () => {
    sessionStore.ingestEvent({ type: "text_delta", payload: { text: "orphan" } });
    expect(selectTranscript(sessionStore.getState(), CONVERSATION)).toEqual([]);
  });

  it("keeps conversations separate", () => {
    startRun();
    sessionStore.ingestEvent({ type: "text_delta", payload: { text: "hello" } });
    expect(selectTranscript(sessionStore.getState(), "conv-2")).toEqual([]);
  });
});

describe("countDiffLines", () => {
  it("counts added and removed lines, skipping file headers", () => {
    const diff = ["--- a/file", "+++ b/file", "@@ -1 +1,2 @@", "-gone", "+here", "+also"].join("\n");
    expect(countDiffLines(diff)).toEqual({ plus: 2, minus: 1 });
  });
});
