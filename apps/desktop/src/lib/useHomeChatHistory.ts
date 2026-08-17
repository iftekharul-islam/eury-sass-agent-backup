import { useCallback, useEffect, useRef, useState } from 'react';
import {
  conversationsApi,
  type ConversationSummary,
  type StoredMessage,
  type UserMessageMeta,
} from './conversations';

const SYNC_DEBOUNCE_MS = 400;

export function useHomeChatHistory(isAuthenticated: boolean) {
  const [conversations, setConversations] = useState<ConversationSummary[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [messages, setMessages] = useState<StoredMessage[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const syncTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  const refreshList = useCallback(async () => {
    // Signed out there is no history to show — an empty list is the truth,
    // not a placeholder one.
    if (!isAuthenticated) {
      setConversations([]);
      setError(null);
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const page = await conversationsApi.listGlobalChats();
      setConversations(page.items);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Failed to load chats');
      setConversations([]);
    } finally {
      setLoading(false);
    }
  }, [isAuthenticated]);

  useEffect(() => {
    void refreshList();
  }, [refreshList]);

  const selectConversation = useCallback(
    async (id: string) => {
      setActiveId(id);
      setLoading(true);
      try {
        const loaded = await conversationsApi.fetchMessages(id);
        setMessages(loaded);
      } catch (e) {
        setError(e instanceof Error ? e.message : 'Failed to load messages');
      } finally {
        setLoading(false);
      }
    },
    [],
  );

  const clearActive = useCallback(() => {
    setActiveId(null);
    setMessages([]);
  }, []);

  const scheduleSync = useCallback(
    (conversationId: string, nextMessages: StoredMessage[], title?: string) => {
      if (syncTimer.current) clearTimeout(syncTimer.current);
      syncTimer.current = setTimeout(() => {
        void conversationsApi.syncMessages(conversationId, nextMessages, title).catch(() => {
          // Offline sync failures are non-fatal for FF
        });
      }, SYNC_DEBOUNCE_MS);
    },
    [],
  );

  const createConversation = useCallback(
    async (model: string, modelColor?: string) => {
      const id = crypto.randomUUID();
      if (!isAuthenticated) {
        setActiveId(id);
        setMessages([]);
        return id;
      }
      const created = await conversationsApi.createChat({ id, model, modelColor });
      setConversations((prev) => [created, ...prev]);
      setActiveId(created.id);
      setMessages([]);
      return created.id;
    },
    [isAuthenticated],
  );

  const appendMessages = useCallback(
    (conversationId: string, userText: string, assistantText: string, model: string) => {
      setMessages((prev) => {
        const nextId = prev.length > 0 ? Math.max(...prev.map((m) => m.id)) + 1 : 1;
        const next: StoredMessage[] = [
          ...prev,
          { id: nextId, role: 'user', content: userText },
          {
            id: nextId + 1,
            role: 'assistant',
            content: assistantText,
            model,
          },
        ];
        scheduleSync(conversationId, next);
        return next;
      });
    },
    [scheduleSync],
  );

  const updateLastAssistant = useCallback(
    (conversationId: string, delta: string, model: string) => {
      setMessages((prev) => {
        if (prev.length === 0) return prev;
        const last = prev[prev.length - 1];
        if (last?.role !== 'assistant') {
          const nextId = Math.max(...prev.map((m) => m.id), 0) + 1;
          const next: StoredMessage[] = [
            ...prev,
            { id: nextId, role: 'assistant', content: delta, model, createdAt: Date.now() },
          ];
          scheduleSync(conversationId, next);
          return next;
        }
        const next = [...prev];
        next[next.length - 1] = {
          ...last,
          content: last.content + delta,
          model,
        };
        scheduleSync(conversationId, next);
        return next;
      });
    },
    [scheduleSync],
  );

  const setLastAssistantContent = useCallback(
    (
      conversationId: string,
      content: string,
      model: string,
      generatedImages?: StoredMessage['generatedImages'],
    ) => {
      setMessages((prev) => {
        if (prev.length === 0) return prev;
        const last = prev[prev.length - 1];
        if (last?.role !== 'assistant') return prev;
        const next = [...prev];
        next[next.length - 1] = {
          ...last,
          content,
          model,
          ...(generatedImages?.length ? { generatedImages } : { generatedImages: undefined }),
        };
        scheduleSync(conversationId, next);
        return next;
      });
    },
    [scheduleSync],
  );

  const ensureAssistantTurn = useCallback((model: string) => {
    setMessages((prev) => {
      const last = prev[prev.length - 1];
      if (last?.role === 'assistant' && last.content === '') {
        return prev;
      }
      const nextId = prev.length > 0 ? Math.max(...prev.map((m) => m.id)) + 1 : 1;
      return [...prev, { id: nextId, role: 'assistant', content: '', model, createdAt: Date.now() }];
    });
  }, []);

  const beginTurn = useCallback(
    (text: string, modelId: string, modelLabel?: string) => {
      setMessages((prev) => {
        const withoutEmptyAssistant =
          prev.length > 0 &&
          prev[prev.length - 1]?.role === 'assistant' &&
          !prev[prev.length - 1]?.content.trim()
            ? prev.slice(0, -1)
            : prev;
        const nextId =
          withoutEmptyAssistant.length > 0
            ? Math.max(...withoutEmptyAssistant.map((m) => m.id)) + 1
            : 1;
        const now = Date.now();
        return [
          ...withoutEmptyAssistant,
          {
            id: nextId,
            role: 'user' as const,
            content: text,
            createdAt: now,
          },
          {
            id: nextId + 1,
            role: 'assistant' as const,
            content: '',
            model: modelId,
            modelLabel,
            createdAt: now,
          },
        ];
      });
    },
    [],
  );

  const addUserMessage = useCallback((text: string, meta?: UserMessageMeta) => {
    setMessages((prev) => {
      const withoutEmptyAssistant =
        prev.length > 0 &&
        prev[prev.length - 1]?.role === 'assistant' &&
        !prev[prev.length - 1]?.content.trim()
          ? prev.slice(0, -1)
          : prev;
      const nextId =
        withoutEmptyAssistant.length > 0
          ? Math.max(...withoutEmptyAssistant.map((m) => m.id)) + 1
          : 1;
      return [
        ...withoutEmptyAssistant,
        {
          id: nextId,
          role: 'user' as const,
          content: text,
          createdAt: Date.now(),
          ...(meta?.modelId ? { model: meta.modelId } : {}),
          ...(meta?.modelLabel ? { modelLabel: meta.modelLabel } : {}),
        },
      ];
    });
  }, []);

  const removeLastAssistantTurn = useCallback(() => {
    setMessages((prev) => {
      const last = prev[prev.length - 1];
      if (last?.role !== 'assistant') return prev;
      return prev.slice(0, -1);
    });
  }, []);

  return {
    conversations,
    activeId,
    messages,
    loading,
    error,
    refreshList,
    selectConversation,
    clearActive,
    createConversation,
    appendMessages,
    updateLastAssistant,
    setLastAssistantContent,
    ensureAssistantTurn,
    beginTurn,
    addUserMessage,
    removeLastAssistantTurn,
    setMessages,
  };
}
