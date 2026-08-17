import { describe, expect, it } from "vitest";
import { stripAnsi } from "./ansi";

const ESC = String.fromCharCode(27);

describe("stripAnsi", () => {
  it("removes CSI colour sequences", () => {
    const raw = `${ESC}[33mwarning${ESC}[39m: missing module`;
    expect(stripAnsi(raw)).toBe("warning: missing module");
  });

  it("leaves plain text unchanged", () => {
    expect(stripAnsi("UNRESOLVED_IMPORT Could not resolve 'vite'")).toBe(
      "UNRESOLVED_IMPORT Could not resolve 'vite'",
    );
  });
});
