/**
 * Hides tool-call syntax from the transcript.
 *
 * # Why this exists
 *
 * The core recovers tool calls from ` ```tool_call ` fences in the assistant's
 * text and strips them before feeding the text back into the next round — but
 * the desktop renders the `text_delta` stream as it arrives, long before that
 * happens. So the raw `{"name":"write_file","arguments":{…}}` block was shown
 * to the user, half-written, as the model typed it.
 *
 * The tool card already shows what was called and what it returned, so the
 * fence is noise. This mirrors the shapes `agent_core::tool_calls` accepts, and
 * also hides a fence that is still streaming and has no closing marker yet.
 */

const CALL_LANGS = new Set(["tool_call", "tool"]);

/**
 * The core's tool names, mirrored from `agent_tools::registry_factory`. A
 * model may fence a call under the tool's own name (```write_file), which the
 * core executes — so the transcript has to hide those too.
 */
const TOOL_NAMES = new Set([
  "read_file",
  "write_file",
  "edit_file",
  "list_dir",
  "glob",
  "grep",
  "run_command",
]);

/**
 * Consumes a balanced `{…}` starting at `from`, mirroring the core's
 * `first_json_object`. Returns the index just past it, or -1.
 *
 * The bracket form is `[tool_call name=…] {…}`: dropping only the marker left
 * the argument object sitting in the transcript as bare text — which is what
 * the user saw after a write_file call.
 */
function endOfJsonObject(text: string, from: number): number {
  const start = text.indexOf("{", from);
  if (start === -1 || text.slice(from, start).trim() !== "") return -1;

  let depth = 0;
  let inString = false;
  let escaped = false;

  for (let i = start; i < text.length; i += 1) {
    const char = text[i];
    if (inString) {
      if (escaped) escaped = false;
      else if (char === "\\") escaped = true;
      else if (char === '"') inString = false;
      continue;
    }
    if (char === '"') inString = true;
    else if (char === "{") depth += 1;
    else if (char === "}") {
      depth -= 1;
      if (depth === 0) return i + 1;
    }
  }
  // Unterminated: it is still streaming, so swallow the rest.
  return text.length;
}

/** Removes `[tool_call …]` markers together with the JSON object each carries. */
function stripBracketCalls(text: string): string {
  const marker = /\[tool_call[^\]]*\]/;
  let out = text;
  let match = marker.exec(out);

  while (match) {
    const afterMarker = match.index + match[0].length;
    const end = endOfJsonObject(out, afterMarker);
    out = out.slice(0, match.index) + out.slice(end === -1 ? afterMarker : end);
    match = marker.exec(out);
  }

  return out;
}

/** Does this fence body look like a tool call rather than data the user wanted? */
function bodyIsCall(body: string): boolean {
  const trimmed = body.trim();
  if (!trimmed.startsWith("{") && !trimmed.startsWith("[")) return false;

  try {
    const parsed: unknown = JSON.parse(trimmed);
    const items = Array.isArray(parsed) ? parsed : [parsed];
    return items.some(
      (item) =>
        !!item &&
        typeof item === "object" &&
        ["name", "tool", "tool_name"].some((key) => key in (item as Record<string, unknown>)),
    );
  } catch {
    // Still streaming: judge the fragment on its shape instead.
    return /"(name|tool|tool_name)"\s*:/.test(trimmed);
  }
}

function isCallFence(lang: string, body: string): boolean {
  const normalized = lang.trim().toLowerCase();
  if (CALL_LANGS.has(normalized)) return true;
  // ```write_file → the body is that tool's arguments.
  if (TOOL_NAMES.has(normalized)) return true;
  if (normalized === "json" || normalized === "") return bodyIsCall(body);
  return false;
}

export function stripToolCallFences(text: string): string {
  if (!text.includes("```") && !text.includes("[tool_call")) return text;

  let out = "";
  let index = 0;

  while (index < text.length) {
    const open = text.indexOf("```", index);
    if (open === -1) {
      out += text.slice(index);
      break;
    }

    out += text.slice(index, open);

    const langEnd = text.indexOf("\n", open + 3);
    if (langEnd === -1) {
      // An opening fence with no newline yet — mid-stream, keep it hidden if
      // it announces a tool call, otherwise show it.
      const lang = text.slice(open + 3).trim().toLowerCase();
      if (!CALL_LANGS.has(lang) && !TOOL_NAMES.has(lang)) out += text.slice(open);
      break;
    }

    const lang = text.slice(open + 3, langEnd);
    const close = text.indexOf("```", langEnd + 1);
    const body = text.slice(langEnd + 1, close === -1 ? text.length : close);

    if (isCallFence(lang, body)) {
      // Drop it. An unterminated fence swallows the rest of the stream, which
      // is what we want while the call is still being typed.
      if (close === -1) break;
      index = close + 3;
      continue;
    }

    if (close === -1) {
      out += text.slice(open);
      break;
    }
    out += text.slice(open, close + 3);
    index = close + 3;
  }

  return stripBracketCalls(out)
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}
