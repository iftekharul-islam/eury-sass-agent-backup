/** Parse AgentEvent payloads emitted from the Rust runtime. */
export type ParsedStreamEvent =
  | { kind: 'text_delta'; text: string }
  | { kind: 'thinking_delta'; text: string }
  | { kind: 'run_complete'; stopReason?: string }
  | { kind: 'run_error'; code?: string; message?: string }
  | { kind: 'tool_start'; toolCallId: string; name: string; arguments: unknown }
  | { kind: 'tool_end'; toolCallId: string; result: unknown }
  | { kind: 'tool_output_delta'; toolCallId: string; stream: 'stdout' | 'stderr'; text: string }
  | { kind: 'approval_required'; toolCallId: string; name: string; arguments: unknown }
  | {
      kind: 'cost_update';
      tokensPrompt: number;
      tokensCompletion: number;
      costUsdMicros: number;
    }
  | { kind: 'context_warning'; tokens: number; limit: number; message: string }
  | { kind: 'meta' }
  | { kind: 'other' };

type RawEvent = {
  type?: string;
  payload?: Record<string, unknown>;
  text?: string;
  run_id?: string;
  code?: string;
  message?: string;
  tool_call_id?: string;
  name?: string;
  arguments?: unknown;
  result?: unknown;
};

function firstStringFrom(
  source: Record<string, unknown> | undefined,
  keys: string[],
): string | undefined {
  if (!source) return undefined;
  for (const key of keys) {
    const value = source[key];
    if (typeof value === "string" && value.length > 0) return value;
  }
  return undefined;
}

function payloadOf(ev: RawEvent): Record<string, unknown> {
  if (ev.payload && typeof ev.payload === 'object') {
    return ev.payload;
  }
  return ev as Record<string, unknown>;
}

function toolCallIdFrom(p: Record<string, unknown>, raw: RawEvent): string {
  return (
    firstStringFrom(p, ['tool_call_id', 'toolCallId']) ??
    firstStringFrom(raw, ['tool_call_id', 'toolCallId']) ??
    ''
  );
}

export function parseStreamEvent(ev: unknown): ParsedStreamEvent {
  const raw = ev as RawEvent;
  const type = raw.type ?? '';
  const p = payloadOf(raw);

  switch (type) {
    case 'text_delta':
      return { kind: 'text_delta', text: String(p.text ?? raw.text ?? '') };
    case 'thinking_delta':
      return { kind: 'thinking_delta', text: String(p.text ?? raw.text ?? '') };
    case 'run_complete':
      return {
        kind: 'run_complete',
        stopReason: String(p.stop_reason ?? p.stopReason ?? 'stop'),
      };
    case 'run_error':
      return {
        kind: 'run_error',
        code: String(p.code ?? raw.code ?? 'EURY_UNKNOWN'),
        message: String(p.message ?? raw.message ?? 'Run failed'),
      };
    case 'tool_start':
      return {
        kind: 'tool_start',
        toolCallId: toolCallIdFrom(p, raw),
        name: String(p.name ?? raw.name ?? ''),
        arguments: p.arguments ?? raw.arguments,
      };
    case 'tool_end':
      return {
        kind: 'tool_end',
        toolCallId: toolCallIdFrom(p, raw),
        result: p.result ?? raw.result,
      };
    case 'tool_output_delta':
      return {
        kind: 'tool_output_delta',
        toolCallId: toolCallIdFrom(p, raw),
        stream: (p.stream ?? raw.stream) === 'stderr' ? 'stderr' : 'stdout',
        text: String(p.text ?? raw.text ?? ''),
      };
    case 'approval_required':
      return {
        kind: 'approval_required',
        toolCallId: toolCallIdFrom(p, raw),
        name: String(p.name ?? raw.name ?? ''),
        arguments: p.arguments ?? raw.arguments,
      };
    case 'cost_update':
      return {
        kind: 'cost_update',
        tokensPrompt: Number(p.tokens_prompt ?? 0),
        tokensCompletion: Number(p.tokens_completion ?? 0),
        costUsdMicros: Number(p.cost_usd_micros ?? 0),
      };
    case 'context_warning':
      return {
        kind: 'context_warning',
        tokens: Number(p.tokens ?? 0),
        limit: Number(p.limit ?? 0),
        message: String(p.message ?? ''),
      };
    case 'meta':
      return { kind: 'meta' };
    default:
      return { kind: 'other' };
  }
}
