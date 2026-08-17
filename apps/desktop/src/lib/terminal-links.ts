/**
 * Clickable URLs in the terminal.
 *
 * xterm ships no link handling of its own, so a dev server's "Local:
 * http://localhost:5173/" was plain text: selectable, not openable. This adds
 * a link provider over the raw core rather than pulling in the web-links
 * addon, because the addon would be a new dependency for a regex and a click
 * handler.
 */
import type { IBufferRange, IDisposable, Terminal } from "@xterm/xterm";
import { isOpenableUrl, openExternalUrl } from "./open";

const URL_PATTERN = /https?:\/\/[^\s"'`<>()[\]{}\\^|]+/g;

/** Punctuation that ends the sentence rather than the URL. */
const TRAILING = /[.,;:!?'"]+$/;

export interface FoundLink {
  text: string;
  /** Index of the first character within the logical (unwrapped) line. */
  start: number;
  end: number;
}

/** Every URL in a logical line, with trailing prose punctuation trimmed off. */
export function findUrls(line: string): FoundLink[] {
  const found: FoundLink[] = [];

  for (const match of line.matchAll(URL_PATTERN)) {
    const raw = match[0];
    const text = raw.replace(TRAILING, "");
    if (!text || !isOpenableUrl(text)) continue;
    const start = match.index ?? 0;
    found.push({ text, start, end: start + text.length - 1 });
  }

  return found;
}

/**
 * Maps a span of a logical line back to buffer coordinates. A wrapped line is
 * one logical string across several rows of exactly `cols` cells, so the row
 * is the quotient and the column the remainder — both 1-based, as xterm wants.
 */
export function toBufferRange(
  link: FoundLink,
  firstRow: number,
  cols: number,
): IBufferRange {
  return {
    start: { x: (link.start % cols) + 1, y: firstRow + Math.floor(link.start / cols) + 1 },
    end: { x: (link.end % cols) + 1, y: firstRow + Math.floor(link.end / cols) + 1 },
  };
}

/**
 * The logical line `row` belongs to: a URL that wrapped across rows is one
 * link, not one broken link per row.
 */
function logicalLine(
  term: Terminal,
  row: number,
): { text: string; firstRow: number } | null {
  const buffer = term.buffer.active;
  if (!buffer.getLine(row)) return null;

  let first = row;
  while (first > 0 && buffer.getLine(first)?.isWrapped) first -= 1;

  let last = row;
  while (last + 1 < buffer.length && buffer.getLine(last + 1)?.isWrapped) last += 1;

  let text = "";
  for (let i = first; i <= last; i += 1) {
    // Untrimmed: every wrapped row is exactly `cols` wide, which is what makes
    // the index arithmetic in `toBufferRange` hold.
    text += buffer.getLine(i)?.translateToString(false) ?? "";
  }

  return { text, firstRow: first };
}

export function registerTerminalLinks(term: Terminal): IDisposable {
  return term.registerLinkProvider({
    provideLinks(bufferLineNumber, callback) {
      const line = logicalLine(term, bufferLineNumber - 1);
      if (!line) return callback(undefined);

      const links = findUrls(line.text).map((link) => ({
        text: link.text,
        range: toBufferRange(link, line.firstRow, term.cols),
        decorations: { pointerCursor: true, underline: true },
        activate: (_event: MouseEvent, text: string) => {
          void openExternalUrl(text).catch((err) => {
            console.error("Failed to open terminal link:", err);
          });
        },
      }));

      callback(links.length > 0 ? links : undefined);
    },
  });
}
