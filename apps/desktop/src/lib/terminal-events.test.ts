import { describe, it, expect } from "vitest";
import { parseTerminalFrame } from "./terminal-events";

describe("parseTerminalFrame", () => {
  it("parses a data frame with its byte payload", () => {
    const raw = { type: "data", payload: { seq: 3, dropped_before: 12, bytes: [104, 105] } };
    const frame = parseTerminalFrame(raw);
    expect(frame.kind).toBe("data");
    if (frame.kind === "data") {
      expect(frame.seq).toBe(3);
      expect(frame.droppedBefore).toBe(12);
      expect(Array.from(frame.bytes)).toEqual([104, 105]);
    }
  });

  it("parses an exited frame", () => {
    const frame = parseTerminalFrame({ type: "exited", payload: { code: 0 } });
    expect(frame).toEqual({ kind: "exited", code: 0 });
  });

  it("parses a killed frame with no payload", () => {
    const frame = parseTerminalFrame({ type: "killed" });
    expect(frame).toEqual({ kind: "killed" });
  });

  it("parses a degraded frame", () => {
    const frame = parseTerminalFrame({
      type: "degraded",
      payload: { reason: "ConPtyUnavailable", detail: "resize may misbehave" },
    });
    expect(frame).toEqual({
      kind: "degraded",
      reason: "ConPtyUnavailable",
      detail: "resize may misbehave",
    });
  });

  it("returns unknown for an unrecognized type", () => {
    expect(parseTerminalFrame({ type: "something_else" })).toEqual({ kind: "unknown" });
  });

  it("returns unknown for malformed input without throwing", () => {
    expect(parseTerminalFrame(null)).toEqual({ kind: "unknown" });
    expect(parseTerminalFrame(undefined)).toEqual({ kind: "unknown" });
    expect(parseTerminalFrame("not an object")).toEqual({ kind: "unknown" });
    expect(parseTerminalFrame(42)).toEqual({ kind: "unknown" });
  });

  it("defaults missing data fields rather than throwing", () => {
    const frame = parseTerminalFrame({ type: "data", payload: {} });
    expect(frame.kind).toBe("data");
    if (frame.kind === "data") {
      expect(frame.seq).toBe(0);
      expect(frame.droppedBefore).toBe(0);
      expect(frame.bytes.length).toBe(0);
    }
  });
});
