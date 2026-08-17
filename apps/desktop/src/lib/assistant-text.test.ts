import { describe, expect, it } from "vitest";
import { stripToolCallFences } from "./assistant-text";

describe("stripToolCallFences", () => {
  it("hides the fence the core executes", () => {
    const text = 'Let me look.\n\n```tool_call\n{"name":"list_dir","arguments":{"path":"."}}\n```';
    expect(stripToolCallFences(text)).toBe("Let me look.");
  });

  it("hides a json fence that is really a call", () => {
    // Exactly what showed up in the app: a write_file call rendered as a code
    // block, mid-stream, with its closing fence still to come.
    const text =
      'Working on it.\n\n```json\n{"name":"write_file","arguments":{"path":"README.md","offset":1,"limit":5000}}';
    expect(stripToolCallFences(text)).toBe("Working on it.");
  });

  it("hides a batched array of calls", () => {
    const text =
      '```json\n[{"name":"run_command","arguments":{"command":"ls"}},{"name":"list_dir","arguments":{}}]\n```\nDone.';
    expect(stripToolCallFences(text)).toBe("Done.");
  });

  it("hides an unterminated fence while it is still streaming", () => {
    const text = 'One moment.\n\n```tool_call\n{"name":"read_file","argu';
    expect(stripToolCallFences(text)).toBe("One moment.");
  });

  it("hides the bracket form together with its arguments", () => {
    // Exactly what leaked into the transcript after a write: the marker was
    // dropped but its argument object stayed behind as bare text.
    const text =
      'Writing it now.\n[tool_call name="write_file"] {"path":"/Users/manna/Documents/test/hello.txt","content":"Hello, world!\\n"}';
    expect(stripToolCallFences(text)).toBe("Writing it now.");
  });

  it("hides a fence named after the tool", () => {
    const text = 'On it.\n\n```write_file\n{"path":"hello.txt","content":"Hello, world!"}\n```';
    expect(stripToolCallFences(text)).toBe("On it.");
  });

  it("hides a tool-named fence that is still streaming", () => {
    const text = 'On it.\n\n```write_file\n{"path":"hello.txt","conte';
    expect(stripToolCallFences(text)).toBe("On it.");
  });

  it("hides several bracket calls in one turn", () => {
    const text =
      '[tool_call name="list_dir"] {"path":"."}\n[tool_call name="read_file"] {"path":"a.rs"}\nDone.';
    expect(stripToolCallFences(text)).toBe("Done.");
  });

  it("keeps braces inside a string from ending the object early", () => {
    const text = '[tool_call name="write_file"] {"content":"a } b","path":"x.txt"}\nSaved.';
    expect(stripToolCallFences(text)).toBe("Saved.");
  });

  it("keeps code the user actually asked for", () => {
    const text = 'Here is the config:\n\n```json\n{"port": 3000, "host": "localhost"}\n```';
    expect(stripToolCallFences(text)).toBe(text.trim());
  });

  it("keeps ordinary fenced code untouched", () => {
    const text = 'Try this:\n\n```ts\nconst name = "eury";\n```\n\nThat should work.';
    expect(stripToolCallFences(text)).toBe(text.trim());
  });

  it("leaves plain prose alone", () => {
    expect(stripToolCallFences("Just a sentence.")).toBe("Just a sentence.");
  });

  it("collapses to nothing when the turn is only a call", () => {
    expect(stripToolCallFences('```tool_call\n{"name":"list_dir","arguments":{}}\n```')).toBe("");
  });
});
