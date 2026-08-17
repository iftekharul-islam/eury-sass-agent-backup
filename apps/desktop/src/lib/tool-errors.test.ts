import { describe, expect, it } from "vitest";
import { extractToolError, formatToolError } from "./tool-errors";

describe("tool-errors", () => {
  it("formats edit ambiguous errors for display", () => {
    const raw =
      "Execution failed: EURY_TOOL_EDIT_AMBIGUOUS: oldString found multiple times, not unique";
    expect(formatToolError(raw)).toContain("appears more than once");
    expect(formatToolError(raw)).toContain("not unique");
  });

  it("extracts error from tool result objects", () => {
    expect(
      extractToolError({
        error: "Execution failed: EURY_TOOL_EDIT_AMBIGUOUS: oldString found multiple times, not unique",
      }),
    ).toContain("appears more than once");
  });

  it("extracts error from string tool results", () => {
    expect(
      extractToolError("Execution failed: EURY_TOOL_EDIT_AMBIGUOUS: oldString not found in file"),
    ).toContain("not found in the file");
  });
});
