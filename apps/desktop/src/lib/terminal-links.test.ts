import { describe, expect, it } from "vitest";
import { findUrls, toBufferRange } from "./terminal-links";
import { isOpenableUrl } from "./open";

describe("findUrls", () => {
  it("finds the URL a dev server prints", () => {
    expect(findUrls("  ➜  Local:   http://localhost:5173/")).toEqual([
      { text: "http://localhost:5173/", start: 14, end: 35 },
    ]);
  });

  it("leaves the sentence's punctuation out of the link", () => {
    const [link] = findUrls("Open https://example.com/docs.");
    expect(link.text).toBe("https://example.com/docs");
  });

  it("finds several URLs on one line", () => {
    const found = findUrls("app http://localhost:3000 api http://localhost:3001");
    expect(found.map((link) => link.text)).toEqual([
      "http://localhost:3000",
      "http://localhost:3001",
    ]);
  });

  it("does not offer to open something the OS should never be handed", () => {
    // The terminal shows program output, so a line of it is untrusted text.
    expect(findUrls("file:///etc/passwd javascript:alert(1) data:text/html,x")).toEqual([]);
    expect(isOpenableUrl("javascript:alert(1)")).toBe(false);
    expect(isOpenableUrl("file:///etc/passwd")).toBe(false);
  });
});

describe("toBufferRange", () => {
  it("maps a link on a single row to 1-based cells", () => {
    const range = toBufferRange({ text: "x", start: 4, end: 9 }, 0, 80);
    expect(range).toEqual({ start: { x: 5, y: 1 }, end: { x: 10, y: 1 } });
  });

  it("carries a wrapped URL onto the row it continues on", () => {
    // 20 columns: index 18 is on the first row, index 24 on the second.
    const range = toBufferRange({ text: "x", start: 18, end: 24 }, 3, 20);
    expect(range).toEqual({ start: { x: 19, y: 4 }, end: { x: 5, y: 5 } });
  });
});
