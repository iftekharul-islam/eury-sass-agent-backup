import { authIpc, handleUnauthorized } from './auth';
import { getAgentApiUrl } from './config';

export interface ConversationSummary {
  id: string;
  title: string;
  variant: 'chat' | 'coding';
  model: string;
  modelColor: string;
  updatedAt: number;
  messageCount: number;
  isPinned: boolean;
  pinnedAt: number | null;
}

export interface GeneratedImagePreview {
  dataUrl: string;
  caption?: string;
  mimeType?: string;
}

export interface StoredMessage {
  id: number;
  role: 'user' | 'assistant';
  content: string;
  model?: string;
  modelLabel?: string;
  color?: string;
  createdAt?: number;
  generatedImages?: GeneratedImagePreview[];
}

interface PaginatedResponse<T> {
  items: T[];
  nextCursor: string | null;
}

type ApiConversationSummary = {
  id: string;
  title: string;
  variant: 'chat' | 'coding';
  model: string;
  modelColor: string;
  updatedAt: string;
  messageCount: number;
  isPinned?: boolean;
  pinnedAt?: string | null;
};

type ApiMessage = {
  role: 'user' | 'assistant';
  content: string;
  model?: string | null;
  color?: string | null;
  sortOrder: number;
  generatedImages?: GeneratedImagePreview[] | null;
};

async function platformFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const tokens = await authIpc.getTokens();
  const response = await fetch(`${getAgentApiUrl()}${path}`, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${tokens.access_token}`,
      ...(init?.headers ?? {}),
    },
  });

  if (response.status === 401) {
    await handleUnauthorized();
    throw new Error('Session expired — please sign in again.');
  }

  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    const message =
      (body as { message?: string })?.message ??
      `Request failed (${response.status})`;
    throw new Error(message);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

function mapSummary(c: ApiConversationSummary): ConversationSummary {
  return {
    id: c.id,
    title: c.title,
    variant: c.variant,
    model: c.model,
    modelColor: c.modelColor,
    updatedAt: new Date(c.updatedAt).getTime(),
    messageCount: c.messageCount,
    isPinned: c.isPinned ?? false,
    pinnedAt: c.pinnedAt ? new Date(c.pinnedAt).getTime() : null,
  };
}

function apiMessagesToStored(messages: ApiMessage[]): StoredMessage[] {
  return messages.map((m) => ({
    id: m.sortOrder + 1,
    role: m.role,
    content: m.content,
    ...(m.model ? { model: m.model } : {}),
    ...(m.color ? { color: m.color } : {}),
    ...(m.generatedImages?.length ? { generatedImages: m.generatedImages } : {}),
  }));
}

function storedMessagesToApi(messages: StoredMessage[]) {
  return messages.map((m) => ({
    role: m.role,
    content: m.content,
    ...(m.model ? { model: m.model } : {}),
    ...(m.color ? { color: m.color } : {}),
    ...(m.generatedImages?.length ? { generatedImages: m.generatedImages } : {}),
  }));
}

export type UserMessageMeta = {
  modelId?: string;
  modelLabel?: string;
};

export const conversationsApi = {
  listGlobalChats: async (cursor?: string | null) => {
    const params = new URLSearchParams({ variant: 'chat', globalOnly: 'true', limit: '20' });
    if (cursor) params.set('cursor', cursor);
    const data = await platformFetch<PaginatedResponse<ApiConversationSummary>>(
      `/conversations?${params.toString()}`,
    );
    return {
      items: data.items.map(mapSummary),
      nextCursor: data.nextCursor,
    };
  },

  createChat: async (input: {
    id?: string;
    model: string;
    modelColor?: string;
    title?: string;
  }) => {
    const data = await platformFetch<ApiConversationSummary>('/conversations', {
      method: 'POST',
      body: JSON.stringify({
        id: input.id,
        variant: 'chat',
        model: input.model,
        modelColor: input.modelColor ?? '#a34054',
        title: input.title,
      }),
    });
    return mapSummary(data);
  },

  fetchMessages: async (conversationId: string) => {
    const data = await platformFetch<PaginatedResponse<ApiMessage>>(
      `/conversations/${conversationId}/messages?limit=100`,
    );
    return apiMessagesToStored(data.items);
  },

  syncMessages: async (
    conversationId: string,
    messages: StoredMessage[],
    title?: string,
  ) => {
    const data = await platformFetch<ApiConversationSummary>(
      `/conversations/${conversationId}/messages`,
      {
        method: 'PUT',
        body: JSON.stringify({
          messages: storedMessagesToApi(messages),
          ...(title ? { title } : {}),
        }),
      },
    );
    return mapSummary(data);
  },
};
