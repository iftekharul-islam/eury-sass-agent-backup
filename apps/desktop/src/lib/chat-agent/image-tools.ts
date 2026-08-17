import { authIpc, handleUnauthorized } from '../auth';
import { getAgentApiUrl } from '../config';
import { humanizeChatError } from '../chat-errors';
import type { ChatToolCall } from './types';
import type { GeneratedImagePreview } from './types';

type ImageSize = '1024x1024' | '1536x1024' | '1024x1536' | '1792x1024' | '1024x1792';

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

async function parseApiError(response: Response): Promise<string> {
  try {
    const data = (await response.json()) as { message?: string | string[]; error?: string };
    if (Array.isArray(data.message)) return data.message[0] ?? 'Request failed.';
    if (typeof data.message === 'string') return data.message;
    if (typeof data.error === 'string') return data.error;
  } catch {
    /* ignore */
  }
  return `Request failed (${response.status})`;
}

export async function generateChatImage(input: {
  prompt: string;
  caption?: string;
  size?: ImageSize;
}): Promise<GeneratedImagePreview & { ok: boolean }> {
  const response = await authorizedFetch('/eury/generate-image', {
    method: 'POST',
    body: JSON.stringify(input),
  });
  if (response.status === 401) {
    await handleUnauthorized();
    throw new Error('Session expired — please sign in again.');
  }
  if (!response.ok) {
    throw new Error(humanizeChatError(new Error(await parseApiError(response))));
  }
  return (await response.json()) as GeneratedImagePreview & { ok: boolean };
}

export async function executeChatImageTool(call: ChatToolCall): Promise<string> {
  const prompt = String(
    call.arguments.prompt ?? call.arguments.description ?? call.arguments.text ?? '',
  ).trim();
  if (!prompt) {
    return JSON.stringify({
      error: 'INVALID_GENERATE_IMAGE',
      message: 'Include non-empty "prompt" in arguments.',
      receivedKeys: Object.keys(call.arguments),
    });
  }
  const caption = String(call.arguments.caption ?? call.arguments.title ?? call.arguments.label ?? '').trim();
  const size = String(call.arguments.size ?? '').trim();
  const result = await generateChatImage({
    prompt,
    ...(caption ? { caption } : {}),
    ...(size ? { size: size as ImageSize } : {}),
  });
  return JSON.stringify(result);
}

export function parseGeneratedImagePreview(content: string): GeneratedImagePreview | null {
  try {
    const parsed = JSON.parse(content) as {
      ok?: boolean;
      dataUrl?: string;
      imageUrl?: string;
      caption?: string;
      mimeType?: string;
    };
    const dataUrl = parsed.dataUrl?.trim() || parsed.imageUrl?.trim();
    if (!parsed.ok || !dataUrl) return null;
    return {
      dataUrl,
      ...(parsed.caption?.trim() ? { caption: parsed.caption.trim() } : {}),
      ...(parsed.mimeType ? { mimeType: parsed.mimeType } : {}),
    };
  } catch {
    return null;
  }
}

export async function searchWeb(query: string): Promise<unknown> {
  const response = await authorizedFetch('/eury/web-search', {
    method: 'POST',
    body: JSON.stringify({ query: query.trim().slice(0, 500) }),
  });
  if (!response.ok) {
    throw new Error(humanizeChatError(new Error(await parseApiError(response))));
  }
  return response.json();
}

export async function executeTavily(
  tool: string,
  arguments_: Record<string, unknown>,
): Promise<unknown> {
  const response = await authorizedFetch('/eury/tavily', {
    method: 'POST',
    body: JSON.stringify({ tool: tool.replace(/^tavily_/, ''), arguments: arguments_ }),
  });
  if (!response.ok) {
    throw new Error(humanizeChatError(new Error(await parseApiError(response))));
  }
  return response.json();
}
