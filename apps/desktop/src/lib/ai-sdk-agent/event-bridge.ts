import { sessionStore } from '../session-store';

export function emitTextDelta(runId: string, text: string) {
  sessionStore.ingestEvent({ type: 'text_delta', payload: { run_id: runId, text } });
}

export function emitThinkingDelta(runId: string, text: string) {
  sessionStore.ingestEvent({
    type: 'thinking_delta',
    payload: { run_id: runId, text },
  });
}

export function emitRunError(runId: string, message: string, code = 'EURY_AGENT_INTERNAL') {
  sessionStore.ingestEvent({
    type: 'run_error',
    payload: { run_id: runId, code, message },
  });
}

export function emitRunComplete(runId: string, stopReason = 'stop') {
  sessionStore.ingestEvent({
    type: 'run_complete',
    payload: { run_id: runId, stop_reason: stopReason },
  });
}
