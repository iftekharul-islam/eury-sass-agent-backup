/** Parses `TerminalFrame` payloads delivered over a terminal session's
 * `tauri::ipc::Channel`. Mirrors `stream-events.ts`'s `parseStreamEvent`
 * shape: a pure function, no React, directly unit-testable.
 *
 * The wire shape today is JSON (`agent_types::terminal::TerminalFrame`,
 * tagged the same way as `AgentEvent`): `{ type, payload }` with `payload`
 * omitted for the unit `killed` variant. `bytes` arrives as a JSON number
 * array, not a raw `ArrayBuffer` — see the note in `terminal.rs` on why
 * that's an interim, correctness-first choice rather than the final
 * throughput-critical encoding.
 */
export type TerminalFrame =
  | { kind: "data"; seq: number; droppedBefore: number; bytes: Uint8Array }
  | { kind: "exited"; code?: number }
  | { kind: "killed" }
  | { kind: "degraded"; reason: string; detail: string }
  | { kind: "unknown" };

interface RawFrame {
  type?: string;
  payload?: {
    seq?: number;
    dropped_before?: number;
    bytes?: number[];
    code?: number;
    reason?: string;
    detail?: string;
  };
}

export function parseTerminalFrame(raw: unknown): TerminalFrame {
  if (raw == null || typeof raw !== "object") {
    return { kind: "unknown" };
  }
  const frame = raw as RawFrame;
  const payload = frame.payload ?? {};

  switch (frame.type) {
    case "data":
      return {
        kind: "data",
        seq: Number(payload.seq ?? 0),
        droppedBefore: Number(payload.dropped_before ?? 0),
        bytes: new Uint8Array(payload.bytes ?? []),
      };
    case "exited":
      return { kind: "exited", code: payload.code };
    case "killed":
      return { kind: "killed" };
    case "degraded":
      return {
        kind: "degraded",
        reason: String(payload.reason ?? ""),
        detail: String(payload.detail ?? ""),
      };
    default:
      return { kind: "unknown" };
  }
}
