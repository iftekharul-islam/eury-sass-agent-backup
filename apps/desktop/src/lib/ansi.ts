/**
 * Strip ANSI/OSC escape sequences from terminal output so tool cards and
 * previews show plain text instead of raw control characters.
 */
const ANSI = new RegExp(
  // eslint-disable-next-line no-control-regex
  "\\u001b\\[[0-9;?]*[ -/]*[@-~]|\\u001b\\][^\\u0007\\u001b]*(?:\\u0007|\\u001b\\\\)",
  "g",
);

export function stripAnsi(text: string): string {
  return text.replace(ANSI, "");
}
