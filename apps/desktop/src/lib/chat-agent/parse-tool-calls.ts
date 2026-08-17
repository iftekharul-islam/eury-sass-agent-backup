import type { ChatToolCall } from './types';

const CHAT_TOOLS = new Set([
  'generate_image',
  'web_search',
  'tavily_search',
  'tavily_extract',
  'tavily_map',
  'tavily_crawl',
  'tavily_research',
]);
const CHAT_TOOL_PATTERN = [...CHAT_TOOLS].join('|');

const FENCE_RE = /```([a-zA-Z0-9_-]*)\s*\n([\s\S]*?)```/g;
const BRACKET_TOOL_CALL_RE = /\[tool_call\s+name=([a-zA-Z0-9_-]+)\]/gi;
const XML_TOOL_CALL_RE = /<tool_call\b[^>]*>([\s\S]*?)<\/tool_call>/gi;
const INCOMPLETE_XML_TOOL_CALL_RE = /<tool_call\b[^>]*>[\s\S]*$/i;
const ORPHAN_XML_TOOL_CALL_TAG_RE = /<\/?tool_call\b[^>]*>/gi;
const NAMED_XML_TOOL_RE = new RegExp(
  `<(${CHAT_TOOL_PATTERN})\\b[^>]*>([\\s\\S]*?)<\\/\\1>`,
  'gi',
);
const INCOMPLETE_NAMED_XML_TOOL_RE = new RegExp(
  `<(?:${CHAT_TOOL_PATTERN})\\b[^>]*>[\\s\\S]*$`,
  'i',
);
const ORPHAN_NAMED_XML_TOOL_TAG_RE = new RegExp(
  `<\\/?(?:${CHAT_TOOL_PATTERN})\\b[^>]*>`,
  'gi',
);
const TOOL_OBJECT_RE = new RegExp(
  `\\{\\s*"name"\\s*:\\s*"(?:${CHAT_TOOL_PATTERN})"\\s*,\\s*"(?:arguments|args)"\\s*:\\s*\\{[\\s\\S]*?\\}\\s*\\}`,
  'g',
);

function loadsJson(raw: string): unknown {
  const text = raw.trim();
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    try {
      return JSON.parse(text.replace(/'/g, '"'));
    } catch {
      return null;
    }
  }
}

function parseArgsObject(raw: string): Record<string, unknown> | null {
  const data = loadsJson(raw);
  if (!data || typeof data !== 'object') return null;
  const record = data as Record<string, unknown>;
  if ('name' in record || 'tool' in record) return null;
  return record;
}

function parseNamedObject(raw: string): ChatToolCall | null {
  const data = loadsJson(raw);
  if (!data || typeof data !== 'object') return null;
  const record = data as Record<string, unknown>;
  const name = record.name ?? record.tool;
  if (typeof name !== 'string' || !CHAT_TOOLS.has(name.trim())) return null;
  const args = record.arguments ?? record.args ?? record.parameters ?? {};
  return {
    name: name.trim(),
    arguments: typeof args === 'object' && args !== null ? (args as Record<string, unknown>) : {},
  };
}

function callFromFence(lang: string, body: string): ChatToolCall | null {
  const normalized = lang.trim().toLowerCase();
  if (normalized === 'tool_call' || normalized === 'json' || normalized === 'tool') {
    return parseNamedObject(body);
  }
  if (CHAT_TOOLS.has(normalized)) {
    const args = loadsJson(body);
    if (args && typeof args === 'object' && !('name' in (args as object))) {
      return { name: normalized, arguments: args as Record<string, unknown> };
    }
    return parseNamedObject(body) ?? { name: normalized, arguments: {} };
  }
  if (!normalized) {
    return parseNamedObject(body);
  }
  return null;
}

function mergeSpans(
  spans: Array<{ start: number; end: number }>,
  next: { start: number; end: number },
): boolean {
  const overlaps = spans.some(
    (span) =>
      (next.start >= span.start && next.start < span.end) ||
      (next.end > span.start && next.end <= span.end),
  );
  if (!overlaps) spans.push(next);
  return !overlaps;
}

export function normalizeChatToolMarkup(text: string): string {
  return text.replace(NAMED_XML_TOOL_RE, (_match, toolName: string, body: string) => {
    const args = body.trim();
    if (!args) return '';
    return `\`\`\`tool_call\n{"name":"${toolName}","arguments":${args}}\n\`\`\``;
  });
}

function parseBracketToolCalls(
  text: string,
): Array<{ call: ChatToolCall; start: number; end: number }> {
  const results: Array<{ call: ChatToolCall; start: number; end: number }> = [];
  for (const match of text.matchAll(BRACKET_TOOL_CALL_RE)) {
    const name = (match[1] ?? '').trim();
    if (!CHAT_TOOLS.has(name)) continue;
    const start = match.index ?? 0;
    const after = text.slice(start + match[0].length);
    const jsonMatch = after.match(/^\s*(\{[\s\S]*?\})\s*/);
    if (!jsonMatch) continue;
    const named = parseNamedObject(jsonMatch[1]!);
    const args = named?.name === name ? named.arguments : parseArgsObject(jsonMatch[1]!);
    if (!args) continue;
    results.push({ call: { name, arguments: args }, start, end: start + match[0].length + jsonMatch[0].length });
  }
  return results;
}

function parseBareToolObjects(
  text: string,
): Array<{ call: ChatToolCall; start: number; end: number }> {
  const results: Array<{ call: ChatToolCall; start: number; end: number }> = [];
  for (const match of text.matchAll(TOOL_OBJECT_RE)) {
    const call = parseNamedObject(match[0]!);
    if (!call) continue;
    results.push({ call, start: match.index ?? 0, end: (match.index ?? 0) + match[0].length });
  }
  return results;
}

function parseXmlToolCalls(
  text: string,
): Array<{ call: ChatToolCall; start: number; end: number }> {
  const results: Array<{ call: ChatToolCall; start: number; end: number }> = [];
  for (const match of text.matchAll(XML_TOOL_CALL_RE)) {
    const body = (match[1] ?? '').trim();
    if (!body) continue;
    const call = parseNamedObject(body);
    if (!call) continue;
    results.push({
      call,
      start: match.index ?? 0,
      end: (match.index ?? 0) + match[0].length,
    });
  }
  return results;
}

export function parseChatToolCalls(text: string): { assistantText: string; toolCalls: ChatToolCall[] } {
  const normalized = normalizeChatToolMarkup(text);
  if (!normalized.trim()) {
    return { assistantText: '', toolCalls: [] };
  }

  const calls: ChatToolCall[] = [];
  const spans: Array<{ start: number; end: number }> = [];

  for (const match of normalized.matchAll(FENCE_RE)) {
    const call = callFromFence(match[1] ?? '', match[2] ?? '');
    if (call && mergeSpans(spans, { start: match.index ?? 0, end: (match.index ?? 0) + match[0].length })) {
      calls.push(call);
    }
  }
  for (const bracket of parseBracketToolCalls(normalized)) {
    if (mergeSpans(spans, { start: bracket.start, end: bracket.end })) calls.push(bracket.call);
  }
  for (const bare of parseBareToolObjects(normalized)) {
    if (mergeSpans(spans, { start: bare.start, end: bare.end })) calls.push(bare.call);
  }
  for (const xml of parseXmlToolCalls(normalized)) {
    if (mergeSpans(spans, { start: xml.start, end: xml.end })) calls.push(xml.call);
  }

  let cleaned = normalized;
  for (const span of [...spans].sort((a, b) => b.start - a.start)) {
    cleaned = cleaned.slice(0, span.start) + cleaned.slice(span.end);
  }
  if (calls.length > 0 || spans.length > 0) {
    cleaned = cleaned.replace(/\n{3,}/g, '\n\n').trim();
  }
  return { assistantText: cleaned, toolCalls: calls };
}

const INCOMPLETE_FENCE_RE = /```(?:tool_call|json|tool|[a-zA-Z0-9_-]*)?\s*\n?[\s\S]*$/i;
const INCOMPLETE_BRACKET_RE = /\[tool_call[\s\S]*$/i;
const INCOMPLETE_TOOL_OBJECT_RE = new RegExp(
  `\\{\\s*"name"\\s*:\\s*"(?:${CHAT_TOOL_PATTERN})"[\\s\\S]*$`,
  'i',
);
const COMPLETE_TOOL_FENCE_RE = /```\s*(?:tool_call|json|tool)\s*\n[\s\S]*?```/gi;
const COMPLETE_NAMED_TOOL_FENCE_RE = new RegExp(
  `\`\`\`\\s*(?:${CHAT_TOOL_PATTERN})\\s*\\n[\\s\\S]*?\`\`\``,
  'gi',
);
const OPENING_TOOL_FENCE_RE = /```\s*(?:tool_call|json|tool)\b[^\n]*\n?[\s\S]*$/gi;
const LONE_TOOL_FENCE_TAG_RE = /```\s*(?:tool_call|json|tool)\b\s*$/gi;
const ORPHAN_TOOL_CALL_LINE_RE = /^\s*(?:`?(?:tool_call|tool)`?\s*)+$/gim;
const TOOL_CALL_LEAK_RE = /\btool_call\b/gi;
const TOOL_CALL_PREFIXES = new Set(['t', 'to', 'too', 'tool', 'tool_', 'tool_c', 'tool_ca', 'tool_cal', 'tool_call', 'json']);

function stripTrailingIncompleteToolJson(text: string): string {
  const lastBrace = text.lastIndexOf('{');
  if (lastBrace < 0) return text;
  const tail = text.slice(lastBrace);
  if (tail.includes('}')) return text;
  const isToolish =
    /^\{\s*$/.test(tail) ||
    /^\{\s*"$/.test(tail) ||
    /^\{\s*"n(?:a(?:m(?:e)?)?)?"?\s*$/i.test(tail) ||
    /^\{\s*"name"\s*:?\s*"?[\s\S]*$/i.test(tail);
  if (!isToolish) return text;
  return text.slice(0, lastBrace).replace(/[ \t]+$/gm, '').trimEnd();
}

export function stripChatToolMarkup(text: string): string {
  const { assistantText } = parseChatToolCalls(text);
  const cleaned = stripTrailingIncompleteToolJson(
    assistantText
      .replace(XML_TOOL_CALL_RE, '')
      .replace(INCOMPLETE_XML_TOOL_CALL_RE, '')
      .replace(ORPHAN_XML_TOOL_CALL_TAG_RE, '')
      .replace(NAMED_XML_TOOL_RE, '')
      .replace(INCOMPLETE_NAMED_XML_TOOL_RE, '')
      .replace(ORPHAN_NAMED_XML_TOOL_TAG_RE, '')
      .replace(COMPLETE_TOOL_FENCE_RE, '')
      .replace(COMPLETE_NAMED_TOOL_FENCE_RE, '')
      .replace(OPENING_TOOL_FENCE_RE, '')
      .replace(LONE_TOOL_FENCE_TAG_RE, '')
      .replace(INCOMPLETE_FENCE_RE, '')
      .replace(INCOMPLETE_BRACKET_RE, '')
      .replace(INCOMPLETE_TOOL_OBJECT_RE, '')
      .replace(ORPHAN_TOOL_CALL_LINE_RE, '')
      .replace(TOOL_CALL_LEAK_RE, '')
      .replace(/\n{3,}/g, '\n\n')
      .trim(),
  );
  const token = cleaned.trim().toLowerCase().replace(/[`*]/g, '');
  if (token.length > 0 && TOOL_CALL_PREFIXES.has(token)) return '';
  return cleaned;
}
