import * as React from "react";
import { Icon } from "./Icons";
import { ChatTurn } from "./chat/ChatTurn";
import { Composer } from "./Composer";
import { useAppSettings, SettingsStore } from "../lib/settings";
import { normalizeEuryProvider, streamChat } from "../lib/chat";
import { humanizeChatError, sanitizeAssistantContent } from "../lib/chat-errors";
import {
  effortToCompressionRatio,
  effortToMaxOutputTokens,
  type EffortLevel,
} from "../lib/effort";
import { useStickToBottom } from "../lib/use-stick-to-bottom";
import { RunStatus } from "./RunStatus";
import type { RunRecord } from "../lib/session-store";
import type { StoredMessage } from "../lib/conversations";

function createHomeRun(conversationId: string, title: string, modelLabel: string): RunRecord {
  const now = Date.now();
  return {
    id: crypto.randomUUID(),
    conversationId,
    title,
    mode: "Chat",
    modelLabel,
    startedAt: now,
    status: "running",
    activityIds: [],
    phase: "starting",
    lastEventAt: now,
    streamedChars: 0,
  };
}

function formatMessageTimestamp(timestamp?: number): string | undefined {
  if (!timestamp) return undefined;
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(timestamp));
}

function resolveMessageModelLabel(
  message: StoredMessage,
  fallbackLabel: string,
): string | undefined {
  return message.modelLabel ?? message.model ?? fallbackLabel;
}

function priorContextMessages(messages: StoredMessage[]): StoredMessage[] {
  return messages.filter(
    (message) =>
      message.content.trim() &&
      (message.role === "user" || message.role === "assistant"),
  );
}

export interface HomeChatCanvasProps {
  conversationId: string;
  messages: StoredMessage[];
  conversationTitle?: string;
  initialPrompt?: string;
  onInitialPromptConsumed?: () => void;
  onBeginTurn: (text: string, modelId: string, modelLabel?: string) => void;
  onAppendAssistantDelta: (conversationId: string, delta: string, model: string) => void;
  onRemoveLastAssistant?: () => void;
  onRunError?: (message: string) => void;
}

export function HomeChatCanvas({
  conversationId,
  messages,
  conversationTitle,
  initialPrompt,
  onInitialPromptConsumed,
  onBeginTurn,
  onAppendAssistantDelta,
  onRemoveLastAssistant,
  onRunError,
}: HomeChatCanvasProps) {
  const [settings] = useAppSettings();
  const [streaming, setStreaming] = React.useState(false);
  const [activeRun, setActiveRun] = React.useState<RunRecord | null>(null);
  const [runError, setRunError] = React.useState<string | null>(null);
  const [effort, setEffort] = React.useState<EffortLevel>("medium");
  const [composerPrefill, setComposerPrefill] = React.useState<{ key: number; text: string } | null>(null);
  const abortRef = React.useRef<AbortController | null>(null);
  const inFlightRef = React.useRef(false);
  const messagesRef = React.useRef(messages);
  messagesRef.current = messages;

  const {
    ref: scrollRef,
    onScroll,
    onWheel,
  } = useStickToBottom<HTMLDivElement>();

  const model = {
    provider: settings.model.activeProvider,
    modelId: settings.model.activeModelId,
    label: settings.model.activeModelLabel,
  };

  const stopStreaming = React.useCallback(() => {
    abortRef.current?.abort();
    abortRef.current = null;
    inFlightRef.current = false;
    setStreaming(false);
    setActiveRun(null);
  }, []);

  const handleSend = React.useCallback(
    async (text: string) => {
      const trimmed = text.trim();
      if (!trimmed || inFlightRef.current) return;

      const last = messagesRef.current[messagesRef.current.length - 1];
      const lastUser = [...messagesRef.current].reverse().find((m) => m.role === "user");
      if (
        lastUser?.content === trimmed &&
        last?.role === "assistant" &&
        (inFlightRef.current || last.content.trim())
      ) {
        return;
      }

      setRunError(null);
      onBeginTurn(trimmed, settings.model.activeModelId, settings.model.activeModelLabel);

      abortRef.current?.abort();
      const controller = new AbortController();
      abortRef.current = controller;
      inFlightRef.current = true;
      setStreaming(true);
      setActiveRun(
        createHomeRun(conversationId, trimmed, settings.model.activeModelLabel),
      );

      const history = priorContextMessages(messagesRef.current);

      try {
        await streamChat(
          {
            text: trimmed,
            contextMessages: history.map((message) => ({
              role: message.role,
              content: message.content,
            })),
            provider: normalizeEuryProvider(settings.model.activeProvider),
            model: settings.model.activeModelId,
            mode: "chat",
            compressionRatio: effortToCompressionRatio(effort),
            maxOutputTokens: effortToMaxOutputTokens(effort),
          },
          (delta) => {
            onAppendAssistantDelta(conversationId, delta, settings.model.activeModelId);
            setActiveRun((prev) =>
              prev
                ? {
                    ...prev,
                    phase: "responding",
                    streamedChars: prev.streamedChars + delta.length,
                    lastEventAt: Date.now(),
                  }
                : prev,
            );
          },
          controller.signal,
        );
      } catch (e) {
        if (controller.signal.aborted) return;
        onRemoveLastAssistant?.();
        const msg = humanizeChatError(e);
        setRunError(msg);
        onRunError?.(msg);
      } finally {
        if (abortRef.current === controller) {
          abortRef.current = null;
        }
        inFlightRef.current = false;
        setStreaming(false);
        setActiveRun(null);
      }
    },
    [
      conversationId,
      effort,
      onAppendAssistantDelta,
      onBeginTurn,
      onRemoveLastAssistant,
      onRunError,
      settings.model.activeModelId,
      settings.model.activeModelLabel,
      settings.model.activeProvider,
    ],
  );

  const handleSendRef = React.useRef(handleSend);
  handleSendRef.current = handleSend;

  React.useEffect(() => {
    const prompt = initialPrompt?.trim();
    if (!prompt) return;

    let active = true;
    void handleSendRef.current(prompt).finally(() => {
      if (active) onInitialPromptConsumed?.();
    });

    return () => {
      active = false;
    };
  }, [conversationId, initialPrompt, onInitialPromptConsumed]);

  const prevConversationIdRef = React.useRef(conversationId);
  React.useEffect(() => {
    if (prevConversationIdRef.current === conversationId) return;
    prevConversationIdRef.current = conversationId;
    abortRef.current?.abort();
    abortRef.current = null;
    inFlightRef.current = false;
    setStreaming(false);
    setActiveRun(null);
  }, [conversationId]);

  const userLabel = settings.profile.preferredName || "You";
  const userAvatar = settings.profile.avatar || settings.profile.preferredName[0] || "U";

  return (
    <div className="chat-surface">
      {conversationTitle ? (
        <div className="pane-head chat-pane-head">
          <h2>{conversationTitle}</h2>
          <Icon name="chev-d" size={14} style={{ color: "var(--color-fg-subtle)" }} />
          <span className="spacer" />
        </div>
      ) : null}

      <div className="convo" ref={scrollRef} onScroll={onScroll} onWheel={onWheel}>
        <div className="convo-col">
          {messages.length === 0 && !streaming && (
            <div className="chat-empty-hint">Start a conversation below.</div>
          )}
          {messages.map((m, index) => {
            const { content, isProviderError } =
              m.role === "assistant"
                ? sanitizeAssistantContent(m.content)
                : { content: m.content, isProviderError: false };

            if (m.role === "assistant" && isProviderError) {
              return (
                <div key={m.id} className="chat-error-banner">
                  {content}
                </div>
              );
            }

            const isStreamingAssistant =
              streaming && m.role === "assistant" && index === messages.length - 1;

            // Retrying an assistant reply resends the prompt that produced
            // it, the same "retry = resend" behavior used for user turns.
            const precedingUserMessage =
              m.role === "assistant"
                ? [...messages.slice(0, index)].reverse().find((msg) => msg.role === "user")
                : undefined;

            return (
              <ChatTurn
                key={m.id}
                id={String(m.id)}
                variant="chat"
                sender={m.role === "user" ? "user" : "assistant"}
                who={m.role === "user" ? userLabel : "Eury"}
                time={formatMessageTimestamp(m.createdAt)}
                modelName={
                  m.role === "assistant"
                    ? resolveMessageModelLabel(m, settings.model.activeModelLabel)
                    : undefined
                }
                avatar={m.role === "user" ? userAvatar : "E"}
                text={content || undefined}
                statusNode={
                  isStreamingAssistant && activeRun ? <RunStatus run={activeRun} /> : undefined
                }
                showActions={false}
                actionsDisabled={streaming}
                onEdit={
                  m.role === "user"
                    ? () => setComposerPrefill({ key: Date.now(), text: m.content })
                    : undefined
                }
                onRetry={
                  m.role === "user"
                    ? () => void handleSend(m.content)
                    : precedingUserMessage
                      ? () => void handleSend(precedingUserMessage.content)
                      : undefined
                }
              />
            );
          })}

          {runError && <div className="chat-error-banner">{runError}</div>}

          <div />
        </div>
      </div>

      <Composer
        mode="Chat"
        picker="effort"
        effort={effort}
        onEffortChange={setEffort}
        model={model}
        onModeChange={() => {}}
        onModelChange={(next) => {
          SettingsStore.updateModel({
            activeProvider: next.provider,
            activeModelId: next.modelId,
            activeModelLabel: next.label,
          });
        }}
        onSubmit={handleSend}
        isRunning={streaming}
        onStop={stopStreaming}
        useManagedModels
        enableImageAttachments={settings.capabilities.enableImageInspection}
        placeholder="Write a message…"
        prefill={composerPrefill}
      />
      <p className="chat-disclaimer">
        Eury is AI and can make mistakes. Please double-check responses.
      </p>
    </div>
  );
}
