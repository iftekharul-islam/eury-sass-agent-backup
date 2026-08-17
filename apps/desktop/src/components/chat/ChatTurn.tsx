import * as React from "react";
import { Icon } from "../Icons";
import { EuryMark } from "../EuryMark";
import { MarkdownRenderer } from "../MarkdownRenderer";
import { sanitizeAssistantContent } from "../../lib/chat-errors";
import type { GeneratedImagePreview } from "../../lib/conversations";
import { ChatGeneratedImages } from "./ChatGeneratedImages";

export interface ChatTurnProps {
  id: string;
  sender: "user" | "assistant";
  who: string;
  time?: string;
  modelName?: string;
  avatar: string;
  badge?: string;
  modeBadge?: string;
  text?: string;
  generatedImages?: GeneratedImagePreview[];
  imageGenerating?: boolean;
  fileChip?: string;
  showActions?: boolean;
  variant?: "chat" | "agent";
  onCopy?: () => void;
  onEdit?: () => void;
  onRetry?: () => void;
  actionsDisabled?: boolean;
  /** Live run status shown under the assistant body while a reply streams. */
  statusNode?: React.ReactNode;
}

const USER_MESSAGE_COLLAPSED_LINES = 6;

function ChatTurnAction({
  label,
  icon,
  onClick,
  disabled,
}: {
  label: string;
  icon: string;
  onClick?: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      className="chat-turn-action"
      title={label}
      aria-label={label}
      onMouseDown={(event) => event.preventDefault()}
      onClick={onClick}
      disabled={disabled || !onClick}
    >
      <Icon name={icon} size={13} />
    </button>
  );
}

function CollapsibleUserMessage({ text }: { text: string }) {
  const [expanded, setExpanded] = React.useState(false);
  const [isLong, setIsLong] = React.useState(false);
  const textRef = React.useRef<HTMLParagraphElement>(null);

  React.useLayoutEffect(() => {
    const el = textRef.current;
    if (!el) return;
    const lineHeight = Number.parseFloat(getComputedStyle(el).lineHeight) || 21.7;
    setIsLong(el.scrollHeight > lineHeight * USER_MESSAGE_COLLAPSED_LINES + 1);
  }, [text]);

  const collapsed = isLong && !expanded;

  return (
    <>
      <p
        ref={textRef}
        className={`chat-user-text${collapsed ? " chat-user-text-collapsed" : ""}`}
      >
        {text}
      </p>
      {isLong ? (
        <button
          type="button"
          className="chat-user-toggle"
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded ? "Show less" : "Show more"}
        </button>
      ) : null}
    </>
  );
}

function CopyAction({
  text,
  disabled,
  className,
}: {
  text: string;
  disabled?: boolean;
  className?: string;
}) {
  const [copied, setCopied] = React.useState(false);

  const copy = async () => {
    if (!text.trim() || disabled) return;
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    } catch {
      /* ignore */
    }
  };

  return (
    <button
      type="button"
      className={`chat-turn-action ${className ?? ""}`.trim()}
      title={copied ? "Copied" : "Copy"}
      aria-label={copied ? "Copied" : "Copy"}
      onMouseDown={(event) => event.preventDefault()}
      onClick={copy}
      disabled={disabled || !text.trim()}
    >
      <Icon name={copied ? "check" : "copy"} size={13} />
    </button>
  );
}

export interface ChatTurnHoverMetaProps {
  modelName?: string;
  time?: string;
  copyText: string;
  onRetry?: () => void;
  onEdit?: () => void;
  align?: "user" | "assistant";
  actionsDisabled?: boolean;
}

/** Model label, timestamp, and turn actions — shown on message hover in chat surfaces. */
export function ChatTurnHoverMeta({
  modelName,
  time,
  copyText,
  onRetry,
  onEdit,
  align = "user",
  actionsDisabled = false,
}: ChatTurnHoverMetaProps) {
  const isAssistant = align === "assistant";

  return (
    <div className={`chat-turn-meta${isAssistant ? " chat-turn-meta-assistant" : ""}`}>
      {modelName || time ? (
        <div className="chat-turn-meta-info">
          {modelName ? <span className="chat-turn-model">{modelName}</span> : null}
          {modelName && time ? <span className="chat-turn-meta-sep">·</span> : null}
          {time ? <span className="chat-turn-date">{time}</span> : null}
        </div>
      ) : null}
      <div className="chat-turn-actions">
        <ChatTurnAction
          label="Retry"
          icon="restore"
          onClick={onRetry}
          disabled={actionsDisabled}
        />
        {!isAssistant ? (
          <ChatTurnAction
            label="Edit"
            icon="pencil"
            onClick={onEdit}
            disabled={actionsDisabled}
          />
        ) : null}
        <CopyAction text={copyText} disabled={actionsDisabled} />
      </div>
    </div>
  );
}

export function ChatTurn({
  sender,
  who,
  time,
  modelName,
  avatar,
  badge,
  modeBadge,
  text,
  generatedImages,
  imageGenerating = false,
  fileChip,
  showActions = true,
  variant = "agent",
  onEdit,
  onRetry,
  actionsDisabled = false,
  statusNode,
}: ChatTurnProps) {
  const isChat = variant === "chat";
  const sanitized = text ? sanitizeAssistantContent(text) : null;
  const displayText = sanitized?.isProviderError ? null : text;

  if (isChat) {
    if (sanitized?.isProviderError) {
      return <div className="chat-error-banner">{sanitized.content}</div>;
    }

    if (sender === "user") {
      return (
        <div className="chat-turn-user-wrap">
          <div className="turn turn-user">
            <div className="turn-body">
              {displayText ? <CollapsibleUserMessage text={displayText} /> : null}
              {fileChip && (
                <span className="chip-file">
                  <Icon name="file" />
                  {fileChip}
                </span>
              )}
            </div>
          </div>
          <ChatTurnHoverMeta
            modelName={modelName}
            time={time}
            copyText={displayText ?? ""}
            onRetry={onRetry}
            onEdit={onEdit}
            actionsDisabled={actionsDisabled}
          />
        </div>
      );
    }

    return (
      <div className="chat-turn-assistant-wrap">
        <div className="turn turn-assistant">
          <div className="turn-body">
            {displayText && <MarkdownRenderer content={displayText} />}
            {imageGenerating ? (
              <div className="chat-image-generating" aria-live="polite">
                Generating image…
              </div>
            ) : null}
            {generatedImages && generatedImages.length > 0 ? (
              <ChatGeneratedImages images={generatedImages} />
            ) : null}
            {fileChip && (
              <span className="chip-file">
                <Icon name="file" />
                {fileChip}
              </span>
            )}
            {statusNode}
          </div>
        </div>
        {displayText || (generatedImages && generatedImages.length > 0) || imageGenerating ? (
          <ChatTurnHoverMeta
            modelName={modelName}
            time={time}
            copyText={displayText ?? ""}
            onRetry={onRetry}
            align="assistant"
            actionsDisabled={actionsDisabled}
          />
        ) : null}
      </div>
    );
  }

  return (
    <div className="turn">
      <div className="turn-head">
        <span className={`avatar ${sender === "user" ? "user" : "eury is-mark"}`}>
          {sender === "user" ? avatar : <EuryMark size={20} />}
        </span>
        <span className="who">{who}</span>
        {badge && <span className="badge">{badge}</span>}
        {modeBadge && <span className="badge">{modeBadge}</span>}
        {time && <span className="when">{time}</span>}

        {showActions && (
          <div className="turn-actions" style={{ marginLeft: "auto", display: "flex", gap: "4px", opacity: 0.6 }}>
            <button type="button" className="btn icon sm ghost" title="Edit and resend" style={{ padding: "4px" }}>
              <Icon name="pencil" />
            </button>
            <button type="button" className="btn icon sm ghost" title="Copy text" style={{ padding: "4px" }}>
              <Icon name="copy" />
            </button>
            <button type="button" className="btn icon sm ghost" title="Retry" style={{ padding: "4px" }}>
              <Icon name="restore" />
            </button>
            <button type="button" className="btn icon sm ghost" title="Delete" style={{ padding: "4px" }}>
              <Icon name="x" />
            </button>
          </div>
        )}
      </div>
      <div className="turn-body">
        {displayText && <MarkdownRenderer content={displayText} />}
        {fileChip && (
          <span className="chip-file">
            <Icon name="file" />
            {fileChip}
          </span>
        )}
      </div>
    </div>
  );
}
