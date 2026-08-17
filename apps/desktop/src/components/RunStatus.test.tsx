/**
 * @vitest-environment jsdom
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { RunStatus } from "./RunStatus";
import type { RunRecord } from "../lib/session-store";

let container: HTMLDivElement;
let root: Root;

function makeRun(overrides: Partial<RunRecord> = {}): RunRecord {
  const now = Date.now();
  return {
    id: "run-1",
    conversationId: "conv-1",
    title: "hello",
    mode: "Agent",
    modelLabel: "GPT 5.6",
    startedAt: now - 13_000,
    status: "running",
    activityIds: [],
    phase: "thinking",
    lastEventAt: now,
    streamedChars: 316,
    ...overrides,
  };
}

function render(props: Partial<React.ComponentProps<typeof RunStatus>> = {}) {
  act(() => {
    root.render(<RunStatus run={props.run ?? makeRun()} {...props} />);
  });
}

beforeEach(() => {
  vi.stubGlobal("IS_REACT_ACT_ENVIRONMENT", true);
  container = document.createElement("div");
  document.body.appendChild(container);
  root = createRoot(container);
});

afterEach(() => {
  act(() => root.unmount());
  container.remove();
  vi.unstubAllGlobals();
});

describe("RunStatus", () => {
  it("reads elapsed · tokens · phase while the model thinks", () => {
    render();
    const text = container.textContent ?? "";

    expect(text).toContain("13s");
    expect(text).toContain("~79 tokens"); // 316 streamed chars ≈ 79 tokens
    expect(text).toContain("thinking…");
  });

  it("prefers the core's own token count over the estimate", () => {
    render({ run: makeRun({ usage: { outputTokens: 1120 } }) });
    const text = container.textContent ?? "";

    expect(text).toContain("1120 tokens");
    expect(text).not.toContain("~");
  });

  it("streams the reasoning as it arrives", () => {
    render({ thinking: "Checking the failing test first" });
    expect(container.querySelector(".run-status-thinking")?.textContent).toContain(
      "Checking the failing test first",
    );
  });

  it("names the tool that is running", () => {
    render({ run: makeRun({ phase: "tool" }), toolName: "run_command" });
    expect(container.textContent).toContain("running run_command…");
  });

  it("asks for approval instead of pretending the tool is already running", () => {
    render({ run: makeRun({ phase: "awaiting_approval" }), toolName: "write_file" });
    const text = container.textContent ?? "";

    expect(text).toContain("waiting for your approval on write_file…");
    expect(text).toContain("approve or deny below");
    expect(text).not.toContain("running a tool");
  });

  it("does not treat an approval wait as a stalled model", () => {
    render({
      run: makeRun({ phase: "awaiting_approval", lastEventAt: Date.now() - 120_000 }),
      toolName: "write_file",
    });
    const text = container.textContent ?? "";

    expect(text).toContain("waiting for your approval");
    expect(text).not.toContain("waiting for the model");
    expect(container.querySelector(".run-status-mark.is-stalled")).toBeNull();
  });

  it("says so when nothing has come back for a long time", () => {
    render({ run: makeRun({ lastEventAt: Date.now() - 120_000 }) });
    const text = container.textContent ?? "";

    expect(text).toContain("waiting for the model…");
    expect(text).toContain("Nothing has come back from the model");
    expect(text).toContain("port 3001");
    expect(container.querySelector(".run-status-mark.is-stalled")).not.toBeNull();
  });

  it("carries no stop control of its own", () => {
    render();
    // Stopping lives in the composer (and Esc); the transcript only reports.
    expect(container.querySelector("button")).toBeNull();
  });
});
