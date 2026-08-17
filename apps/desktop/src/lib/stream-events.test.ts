import { describe, expect, it } from "vitest";
import { parseStreamEvent } from "./stream-events";

describe("parseStreamEvent", () => {
  it("reads approval ids from nested snake_case payloads", () => {
    const parsed = parseStreamEvent({
      type: "approval_required",
      payload: {
        tool_call_id: "call_0_0",
        name: "write_file",
        arguments: { path: "hello.txt", content: "hi" },
      },
    });

    expect(parsed).toEqual({
      kind: "approval_required",
      toolCallId: "call_0_0",
      name: "write_file",
      arguments: { path: "hello.txt", content: "hi" },
    });
  });

  it("reads approval ids from camelCase payloads", () => {
    const parsed = parseStreamEvent({
      type: "approval_required",
      payload: {
        toolCallId: "call_0_1",
        name: "write_file",
        arguments: { path: "hello.txt", content: "hi" },
      },
    });

    expect(parsed.kind).toBe("approval_required");
    if (parsed.kind === "approval_required") {
      expect(parsed.toolCallId).toBe("call_0_1");
    }
  });

  it("parses live command output chunks", () => {
    const parsed = parseStreamEvent({
      type: "tool_output_delta",
      payload: {
        tool_call_id: "run_1",
        stream: "stdout",
        text: "added 42 packages\n",
      },
    });

    expect(parsed).toEqual({
      kind: "tool_output_delta",
      toolCallId: "run_1",
      stream: "stdout",
      text: "added 42 packages\n",
    });
  });
});
