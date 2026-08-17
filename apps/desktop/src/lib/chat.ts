import { authIpc, handleUnauthorized } from './auth';
import { getAgentApiUrl } from './config';
import { humanizeChatError, humanizeProviderErrorText } from './chat-errors';

export type ChatContextMessage = {
  role: 'user' | 'assistant' | 'system';
  content: string;
};

export type EuryModelCatalogItem = {
  id: string;
  provider: string;
  apiProvider: string;
  version: string;
  tier: string;
  color: string;
  mock?: boolean;
};

export type EuryModelsResponse = {
  models: EuryModelCatalogItem[];
  legacyReplacements: Record<string, string>;
};

export type ChatToolResultPayload = {
  role: 'tool';
  name: string;
  content: string;
};

export type ChatStreamRequest = {
  text: string;
  contextMessages?: ChatContextMessage[];
  provider: string;
  model: string;
  mode?: 'chat' | 'coding';
  compressionRatio?: number;
  maxOutputTokens?: number;
  enableImageTool?: boolean;
  toolResults?: ChatToolResultPayload[];
  mock?: boolean;
};

type ChatStreamEvent =
  | { type: 'delta'; text: string }
  | { type: 'done'; text?: string }
  | { type: 'error'; error: string };

/** Same provider names as the web chat API (`Claude`, `OpenAI`, …). */
export function normalizeEuryProvider(provider: string): string {
  const normalized = provider.trim().toLowerCase();
  if (normalized === 'claude' || normalized === 'anthropic') return 'Claude';
  if (normalized === 'google' || normalized === 'gemini') return 'Google';
  if (normalized === 'xai' || normalized === 'grok') return 'xAI';
  return 'OpenAI';
}

async function authorizedFetch(path: string, init?: RequestInit): Promise<Response> {
  const tokens = await authIpc.getTokens();
  return fetch(`${getAgentApiUrl()}${path}`, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${tokens.access_token}`,
      ...(init?.headers ?? {}),
    },
  });
}

export async function getEuryModels(): Promise<EuryModelsResponse> {
  const response = await authorizedFetch('/eury/models');
  if (response.status === 401) {
    await handleUnauthorized();
    throw new Error('Session expired — please sign in again.');
  }
  if (response.status === 401) {
    await handleUnauthorized();
    throw new Error('Session expired — please sign in again.');
  }

  if (!response.ok) {
    throw new Error(`Failed to load models (${response.status})`);
  }
  return response.json() as Promise<EuryModelsResponse>;
}

export async function streamChat(
  request: ChatStreamRequest,
  onDelta: (text: string) => void,
  signal?: AbortSignal,
): Promise<void> {
  const response = await authorizedFetch('/eury/stream', {
    method: 'POST',
    body: JSON.stringify({
      mode: 'chat',
      compressionRatio: 0.78,
      maxOutputTokens: 5000,
      ...request,
      provider: normalizeEuryProvider(request.provider),
    }),
    signal,
  });

  if (response.status === 401) {
    await handleUnauthorized();
    throw new Error('Session expired — please sign in again.');
  }

  if (!response.ok) {
    const body = await response.text();
    let message = `Chat failed (${response.status})`;
    try {
      const parsed = JSON.parse(body) as { message?: string };
      if (parsed.message) message = parsed.message;
    } catch {
      if (body.trim()) message = body.trim();
    }
    throw new Error(humanizeChatError(new Error(message)));
  }

  if (!response.body) {
    throw new Error('Streaming is not supported in this environment.');
  }

  const reader = response.body.pipeThrough(new TextDecoderStream()).getReader();
  let buffer = '';

  const handleDelta = (text: string) => {
    const providerError = humanizeProviderErrorText(text);
    if (providerError) {
      throw new Error(providerError);
    }
    onDelta(text);
  };

  while (true) {
    const { value, done } = await reader.read();
    if (done) break;

    buffer += value;
    const lines = buffer.split('\n');
    buffer = lines.pop() ?? '';

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) continue;

      const event = JSON.parse(trimmed) as ChatStreamEvent;
      if (event.type === 'delta' && event.text) {
        handleDelta(event.text);
      }
      if (event.type === 'error') {
        throw new Error(humanizeChatError(new Error(event.error || 'Chat stream failed')));
      }
    }
  }

  if (buffer.trim()) {
    const event = JSON.parse(buffer.trim()) as ChatStreamEvent;
    if (event.type === 'delta' && event.text) {
      handleDelta(event.text);
    }
    if (event.type === 'error') {
      throw new Error(humanizeChatError(new Error(event.error || 'Chat stream failed')));
    }
  }
}
