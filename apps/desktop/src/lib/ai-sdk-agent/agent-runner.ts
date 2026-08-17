import { streamText, stepCountIs, APICallError, type CoreMessage } from 'ai';
import { invoke } from '@tauri-apps/api/core';
import type { HistoryMessage } from '../ipc';
import { handleUnauthorized } from '../auth';
import { resolveLanguageModel } from './model-registry';
import { buildLocalTools } from './local-tools';
import { buildSystemPrompt, wrapUserMessage } from './system-prompt';
import {
  emitTextDelta,
  emitThinkingDelta,
  emitRunError,
  emitRunComplete,
} from './event-bridge';
import { describeThrown } from '../chat-errors';

export type AiSdkRunOptions = {
  runId: string;
  conversationId: string;
  mode: 'chat' | 'ask' | 'plan' | 'agent' | 'build';
  prompt: string;
  history?: HistoryMessage[];
  provider: string;
  modelId: string;
  workspaceRoot?: string;
  isTrusted?: boolean;
  maxSteps?: number;
  signal?: AbortSignal;
};

const MAX_AGENT_STEPS = 25;

function isOpenAiProvider(provider: string): boolean {
  const p = provider.trim().toLowerCase();
  return p === 'openai' || p.startsWith('gpt');
}

function toCoreMessages(history: HistoryMessage[], userText: string): CoreMessage[] {
  const messages: CoreMessage[] = [];
  for (const turn of history) {
    messages.push({ role: turn.role, content: turn.content });
  }
  messages.push({ role: 'user', content: userText });
  return messages;
}

/** True for a user-initiated stop, not a real failure — the caller already
 * marked the run "cancelled" in the UI, so this must not surface as an error. */
function isAbortError(err: unknown, signal?: AbortSignal): boolean {
  if (signal?.aborted) return true;
  return err instanceof Error && err.name === 'AbortError';
}

export async function runAiSdkAgentTurn(options: AiSdkRunOptions): Promise<void> {
  const maxSteps = options.maxSteps ?? MAX_AGENT_STEPS;
  const hasTools = Boolean(options.workspaceRoot) && options.mode !== 'chat';
  let runFinished = false;

  const finishRunSuccess = async (stopReason = 'stop') => {
    if (runFinished) return;
    runFinished = true;
    emitRunComplete(options.runId, stopReason);
    try {
      await invoke('ai_run_complete', {
        request: {
          run_id: options.runId,
          conversation_id: options.conversationId,
          stop_reason: stopReason,
        },
      });
    } catch {
      // unregister best-effort
    }
  };

  try {
    await invoke('ai_run_register', {
      request: {
        run_id: options.runId,
        conversation_id: options.conversationId,
      },
    });

    const model = await resolveLanguageModel(options.provider, options.modelId);
    const system = buildSystemPrompt({
      mode: options.mode,
      workspaceRoot: options.workspaceRoot,
      isTrusted: options.isTrusted,
      hasTools,
    });

    const userMessage = wrapUserMessage(options.prompt);
    const messages = toCoreMessages(options.history ?? [], userMessage);

    const tools = hasTools
      ? buildLocalTools({
          runId: options.runId,
          conversationId: options.conversationId,
          mode: options.mode,
          workspaceRoot: options.workspaceRoot,
        })
      : undefined;

    let streamError: Error | undefined;
    let hadOutput = false;
    let hadText = false;

    const result = streamText({
      model,
      system,
      messages,
      tools,
      stopWhen: tools ? stepCountIs(maxSteps) : undefined,
      abortSignal: options.signal,
      providerOptions:
        tools && isOpenAiProvider(options.provider)
          ? { openai: { reasoningEffort: 'none' } }
          : undefined,
      onFinish: ({ text }) => {
        if (text && !hadText) {
          hadText = true;
          hadOutput = true;
          emitTextDelta(options.runId, text);
        }
      },
      onError: ({ error }) => {
        streamError = error instanceof Error ? error : new Error(String(error));
      },
    });

    for await (const part of result.fullStream) {
      if (part.type === 'abort') break;

      if (part.type === 'text-delta' && part.text) {
        hadOutput = true;
        hadText = true;
        emitTextDelta(options.runId, part.text);
        continue;
      }

      if (part.type === 'reasoning-delta' && part.text) {
        hadOutput = true;
        emitThinkingDelta(options.runId, part.text);
        continue;
      }

      if (part.type === 'tool-call' || part.type === 'tool-result') {
        hadOutput = true;
        continue;
      }

      if (part.type === 'error') {
        streamError =
          part.error instanceof Error ? part.error : new Error(String(part.error));
      }
    }

    if (options.signal?.aborted) {
      await finishRunSuccess('stop');
      return;
    }

    if (streamError) {
      throw streamError;
    }

    if (!hadOutput) {
      throw new Error(
        'Model returned no response. Check that you are signed in and the backend API keys are configured.',
      );
    }

    await finishRunSuccess();
  } catch (err) {
    if (isAbortError(err, options.signal)) {
      await finishRunSuccess('stop');
      return;
    }
    runFinished = true;
    if (APICallError.isInstance(err) && err.statusCode === 401) {
      await handleUnauthorized();
    }
    const message = describeThrown(err, 'AI SDK run failed');
    emitRunError(options.runId, message);
    try {
      await invoke('ai_run_complete', {
        request: {
          run_id: options.runId,
          conversation_id: options.conversationId,
          stop_reason: 'error',
        },
      });
    } catch {
      // unregister best-effort
    }
    throw err;
  }
}
