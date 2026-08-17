/**
 * @vitest-environment jsdom
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { ConversationCanvas } from "./ConversationCanvas";
import { sessionStore } from "../lib/session-store";
import { previewServers } from "../lib/preview-servers";

const CONVERSATION = "conv-render";

let container: HTMLDivElement;
let root: Root;

function render(props: Partial<React.ComponentProps<typeof ConversationCanvas>> = {}) {
  act(() => {
    root.render(
      <ConversationCanvas
        conversationId={CONVERSATION}
        workspaceRoot="/tmp/project"
        workspaceName="project"
        {...props}
      />,
    );
  });
}

beforeEach(() => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new Error("offline")));
  // jsdom has no layout engine, so the auto-scroll effect needs a stub.
  Element.prototype.scrollIntoView = vi.fn();
  sessionStore.reset();
  previewServers.reset();
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});

describe("ConversationCanvas", () => {
  it("shows an empty state — not a canned transcript — for a new conversation", () => {
    render();
    const text = container.textContent ?? "";

    expect(text).toContain("Ask Eury to work on project");
    // The old mockup transcript must not come back.
    expect(text).not.toContain("pnpm test session");
    expect(text).not.toContain("session.test.ts");
    expect(text).not.toContain("Thought for 2.1s");
    expect(text).not.toContain("Verifying the fix");
    expect(container.querySelectorAll(".card")).toHaveLength(0);
  });

  it("shows thinking immediately when a run starts", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "create a react app",
    });
    sessionStore.startRun({
      runId: "run-immediate-thinking",
      conversationId: CONVERSATION,
      prompt: "create a react app",
      mode: "Agent",
      modelLabel: "GPT 5.6",
      workspaceRoot: "/tmp/project",
    });

    render();
    expect(container.textContent).toContain("Thinking");
    expect(container.textContent).not.toContain("responding…");
  });

  it("shows responding status once answer text starts streaming", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "explain this file",
    });
    sessionStore.startRun({
      runId: "run-responding",
      conversationId: CONVERSATION,
      prompt: "explain this file",
      mode: "Agent",
      modelLabel: "GPT 5.6",
      workspaceRoot: "/tmp/project",
    });
    act(() => {
      sessionStore.ingestEvent({ type: "text_delta", payload: { text: "This file exports" } });
    });

    render();
    expect(container.textContent).toContain("responding…");
    expect(container.textContent).toContain("This file exports");
  });

  it("renders only what the run actually emitted", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "run the linter",
    });
    sessionStore.startRun({
      runId: "run-render",
      conversationId: CONVERSATION,
      prompt: "run the linter",
      mode: "Agent",
      modelLabel: "Opus 5",
      workspaceRoot: "/tmp/project",
      workspaceName: "project",
    });
    act(() => {
      sessionStore.ingestEvent({ type: "text_delta", payload: { text: "Linting now." } });
      sessionStore.ingestEvent({
        type: "tool_start",
        payload: {
          tool_call_id: "tc-1",
          name: "run_command",
          arguments: { command: "pnpm lint" },
        },
      });
    });

    render();
    const text = container.textContent ?? "";

    expect(text).toContain("run the linter");
    expect(text).toContain("Linting now.");
    expect(text).toContain("pnpm lint");
    // Live progress, not an indefinite spinner.
    expect(text).toContain("running run_command…");
    expect(container.querySelectorAll(".card")).toHaveLength(1);
  });

  it("shows an approval prompt instead of a generic running-tool state", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "create hello.txt",
    });
    sessionStore.startRun({
      runId: "run-approval",
      conversationId: CONVERSATION,
      prompt: "create hello.txt",
      mode: "Agent",
      modelLabel: "GPT 5.6",
      workspaceRoot: "/tmp/project",
      workspaceName: "project",
    });
    act(() => {
      sessionStore.ingestEvent({
        type: "approval_required",
        payload: {
          tool_call_id: "tc-write",
          name: "write_file",
          arguments: { path: "hello.txt", content: "Hello, world!\n" },
        },
      });
    });

    render();
    const text = container.textContent ?? "";

    expect(text).toContain("Approval required");
    expect(text).toContain("Hello, world!");
    expect(text).toContain("waiting for your approval on write_file");
    expect(text).not.toContain("running a tool");
    // The prompt belongs to the turn it interrupted, not to a dock parked at
    // the bottom of the window with the transcript stranded above it.
    expect(container.querySelector(".approval-dock")).toBeNull();
    expect(
      container.querySelector(".turn-assistant .card.approve"),
    ).not.toBeNull();
  });

  it("folds the reasoning once the turn has an answer", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "why does this fail?",
    });
    sessionStore.startRun({
      runId: "run-thinking",
      conversationId: CONVERSATION,
      prompt: "why does this fail?",
      mode: "Agent",
      modelLabel: "Opus 5",
      workspaceRoot: "/tmp/project",
    });
    act(() => {
      sessionStore.ingestEvent({
        type: "thinking_delta",
        payload: { text: "The assertion runs before the fixture loads." },
      });
    });

    render();
    // Still reasoning: open, and labelled as live.
    expect(container.querySelector('.thinking-reveal[data-open="true"]')).not.toBeNull();
    expect(container.textContent).toContain("Thinking");

    act(() => {
      sessionStore.ingestEvent({ type: "text_delta", payload: { text: "The fixture is late." } });
      sessionStore.ingestEvent({ type: "run_complete", payload: {} });
    });
    render();

    // Answering closed the window: the reasoning folds behind one line and the
    // answer is what is left on screen.
    expect(container.querySelector('.thinking-reveal[data-open="true"]')).toBeNull();
    expect(container.textContent).toContain("Thought for");
    expect(container.textContent).toContain("The fixture is late.");
  });

  it("offers to run a shell block when a terminal is available", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "how do I start it?",
    });
    sessionStore.startRun({
      runId: "run-shell",
      conversationId: CONVERSATION,
      prompt: "how do I start it?",
      mode: "Agent",
      modelLabel: "Opus 5",
      workspaceRoot: "/tmp/project",
    });
    act(() => {
      sessionStore.ingestEvent({
        type: "text_delta",
        payload: { text: "Run it with:\n\n```bash\nnpm run dev\n```\n" },
      });
      sessionStore.ingestEvent({ type: "run_complete", payload: {} });
    });

    const commands: string[] = [];
    render({ onRunCommand: (command: string) => commands.push(command) });

    const runButton = container.querySelector<HTMLButtonElement>(".code-block-run");
    expect(runButton).not.toBeNull();
    act(() => runButton?.click());
    expect(commands).toEqual(["npm run dev"]);
  });

  it("stops following the stream the moment the user scrolls up", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "explain this",
    });
    sessionStore.startRun({
      runId: "run-scroll",
      conversationId: CONVERSATION,
      prompt: "explain this",
      mode: "Agent",
      modelLabel: "Opus 5",
      workspaceRoot: "/tmp/project",
    });
    render();

    const convo = container.querySelector<HTMLDivElement>(".convo")!;
    // jsdom has no layout, so the scroll geometry is supplied — including a
    // scrollTop that remembers what was written to it.
    let top = 1500;
    Object.defineProperty(convo, "scrollHeight", { value: 2000, configurable: true });
    Object.defineProperty(convo, "clientHeight", { value: 500, configurable: true });
    Object.defineProperty(convo, "scrollTop", {
      configurable: true,
      get: () => top,
      set: (value: number) => {
        top = value;
      },
    });

    const scroll = (to: number) =>
      act(() => {
        convo.scrollTop = to;
        convo.dispatchEvent(new Event("scroll", { bubbles: true }));
      });

    scroll(1500);

    // A small scroll up — 20px, still near the bottom. This used to stay
    // pinned, so the next delta yanked the view back down: the shake.
    scroll(1480);

    act(() => {
      sessionStore.ingestEvent({ type: "text_delta", payload: { text: "More output. " } });
    });
    expect(convo.scrollTop).toBe(1480);

    // Scrolling back to the bottom hands the stream back.
    scroll(1500);

    act(() => {
      sessionStore.ingestEvent({ type: "text_delta", payload: { text: "Still going. " } });
    });
    expect(convo.scrollTop).toBe(2000);
  });

  it("previews the app once a dev server announces itself", () => {
    // Starting a server used to end at a line of terminal output; the running
    // app now shows up in the conversation.
    act(() => {
      previewServers.observeOutput("term-1", "  ➜  Local:   http://localhost:5173/\n");
    });
    render();

    const frame = container.querySelector<HTMLIFrameElement>("iframe.preview-frame");
    expect(frame?.getAttribute("src")).toBe("http://localhost:5173/");
    expect(container.textContent).toContain("localhost:5173");
    expect(container.textContent).toContain("Live");
  });

  it("says a preview is dead once its terminal exits", () => {
    act(() => {
      previewServers.observeOutput("term-1", "http://localhost:5173/");
      previewServers.markTerminalStopped("term-1");
    });
    render();

    expect(container.querySelector("iframe.preview-frame")).toBeNull();
    expect(container.textContent).toContain("no longer running");
  });

  it("shows no preview when nothing is serving", () => {
    render();
    expect(container.querySelector(".preview-card")).toBeNull();
  });

  it("hides the Run affordance when the workspace is untrusted", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "version?",
    });
    sessionStore.startRun({
      runId: "run-shell-untrusted",
      conversationId: CONVERSATION,
      prompt: "version?",
      mode: "Agent",
      modelLabel: "Opus 5",
    });
    act(() => {
      sessionStore.ingestEvent({
        type: "text_delta",
        payload: { text: "```bash\nnode --version\n```\n" },
      });
      sessionStore.ingestEvent({ type: "run_complete", payload: {} });
    });

    render({ workspaceRoot: "/tmp/project", onRunCommand: undefined, isWorkspaceTrusted: false });
    expect(container.querySelector(".code-block-run")).toBeNull();
  });

  it("hides the Run affordance when there is no terminal to run in", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "how do I start it?",
    });
    sessionStore.startRun({
      runId: "run-shell-2",
      conversationId: CONVERSATION,
      prompt: "how do I start it?",
      mode: "Agent",
      modelLabel: "Opus 5",
    });
    act(() => {
      sessionStore.ingestEvent({
        type: "text_delta",
        payload: { text: "```bash\nnpm run dev\n```\n" },
      });
      sessionStore.ingestEvent({ type: "run_complete", payload: {} });
    });

    render({ workspaceRoot: undefined, onRunCommand: undefined });
    expect(container.querySelector(".code-block-run")).toBeNull();
  });

  it("reports a failed run on the turn it belongs to", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "hello",
    });
    sessionStore.startRun({
      runId: "run-failed",
      conversationId: CONVERSATION,
      prompt: "hello",
      mode: "Agent",
      modelLabel: "Opus 5",
      workspaceRoot: "/tmp/project",
    });
    act(() => {
      sessionStore.failRun("run-failed", "A foreground run is already active");
    });

    render();
    const text = container.textContent ?? "";

    // The old behaviour left this turn blank and spun forever.
    expect(text).toContain("A foreground run is already active");
    expect(text).not.toContain("thinking…");
  });

  it("says so when no project is attached, instead of looking ready", () => {
    // With no workspace the run carries no tools at all, and the only hint
    // used to be a placeholder reading "Write a message…".
    render({ workspaceRoot: undefined, workspaceName: undefined, onOpenProject: () => {} });
    const text = container.textContent ?? "";

    expect(text).toContain("No project open");
    expect(text).toContain("Open project");
  });

  it("shows no such notice once a project is attached", () => {
    render();
    expect(container.textContent).not.toContain("No project open");
  });

  it("says the project is untrusted, with a way to fix it", () => {
    // Policy denies every write in an untrusted project; the app used to show
    // no sign of that until a run came back "denied by policy".
    render({ isWorkspaceTrusted: false, onTrustWorkspace: () => {} });
    const text = container.textContent ?? "";

    expect(text).toContain("untrusted");
    expect(text).toContain("Trust project");
  });

  it("shows no trust notice for a trusted project", () => {
    render({ isWorkspaceTrusted: true });
    expect(container.textContent).not.toContain("untrusted");
  });

  it("warns in the composer when the mode cannot change files", () => {
    render({ mode: "Ask" });
    const placeholder = container.querySelector("textarea")?.getAttribute("placeholder");
    expect(placeholder).toContain("Ask mode can't change files");
  });

  it("uses the same transcript shape as the Home chat", () => {
    sessionStore.appendUserTurn({
      conversationId: CONVERSATION,
      who: "You",
      avatar: "U",
      text: "hi",
    });
    render();

    // Home chat markup: a right-aligned user bubble and a bare assistant
    // column — no per-turn avatar/name header.
    expect(container.querySelector(".chat-surface")).not.toBeNull();
    expect(container.querySelector(".chat-turn-user-wrap")).not.toBeNull();
    expect(container.querySelector(".turn-head")).toBeNull();
    expect(container.textContent).toContain("Eury is AI and can make mistakes");
  });
});
