/** Normalize tool failure payloads into text the UI can show. */
export function formatToolError(raw: string): string {
  let text = raw.trim();
  if (!text) return text;

  text = text.replace(/^Execution failed:\s*/i, "");

  if (text.includes("EURY_TOOL_EDIT_AMBIGUOUS")) {
    const detail = text.replace(/^EURY_TOOL_EDIT_AMBIGUOUS:\s*/i, "");
    if (detail.includes("not unique") || detail.includes("multiple times")) {
      return `Edit failed: the text to replace appears more than once. Add more surrounding lines so only one match exists. (${detail})`;
    }
    if (detail.includes("not found")) {
      return `Edit failed: that text was not found in the file. (${detail})`;
    }
    return `Edit failed: ${detail}`;
  }

  if (text.startsWith("EURY_")) {
    const colon = text.indexOf(":");
    if (colon > 0) {
      return text.slice(colon + 1).trim() || text;
    }
  }

  return text;
}

export function extractToolError(result: unknown): string | undefined {
  if (typeof result === "string" && result.trim()) {
    return formatToolError(result);
  }

  if (!result || typeof result !== "object") {
    return undefined;
  }

  const record = result as Record<string, unknown>;
  for (const key of ["error", "message", "stderr", "detail", "reason"]) {
    const value = record[key];
    if (typeof value === "string" && value.trim()) {
      return formatToolError(value);
    }
  }

  if (record.error && typeof record.error === "object") {
    const nested = record.error as Record<string, unknown>;
    for (const key of ["message", "error", "detail"]) {
      const value = nested[key];
      if (typeof value === "string" && value.trim()) {
        return formatToolError(value);
      }
    }
  }

  return undefined;
}
