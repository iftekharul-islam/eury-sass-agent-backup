import * as React from "react";
import { Icon } from "./Icons";
import { ModelPicker } from "./ModelPicker";
import { ModePicker } from "./ModePicker";
import { fileToAttachment } from "../lib/cloud";
import type { EffortLevel } from "../lib/effort";

const TEXTAREA_MIN_HEIGHT = 24;
const TEXTAREA_MAX_LINES = 8;

export interface ModelSelection {
  provider: string;
  modelId: string;
  label: string;
}

export interface ComposerAttachment {
  id: string;
  name: string;
  contentType: string;
  dataBase64: string;
  previewUrl?: string;
}

export interface ComposerProps {
  mode: string;
  model: ModelSelection;
  placeholder?: string;
  disabled?: boolean;
  /** A run is streaming: the send button becomes a stop button. */
  isRunning?: boolean;
  onStop?: () => void;
  useManagedModels?: boolean;
  enableImageAttachments?: boolean;
  picker?: "mode" | "effort";
  effort?: EffortLevel;
  onModeChange: (mode: string) => void;
  onEffortChange?: (effort: EffortLevel) => void;
  onModelChange: (model: ModelSelection) => void;
  onSubmit: (text: string, attachments?: ComposerAttachment[]) => void;
  draftId?: string;
  prefill?: { key: number; text: string } | null;
}

export function Composer({
  mode,
  model,
  placeholder = "Reply to Eury, or ⌘⏎ to steer the run…",
  disabled,
  isRunning = false,
  onStop,
  useManagedModels = true,
  enableImageAttachments = true,
  picker = "mode",
  effort = "medium",
  onModeChange,
  onEffortChange,
  onModelChange,
  onSubmit,
  draftId,
  prefill,
}: ComposerProps) {
  const [text, setText] = React.useState("");
  const textareaRef = React.useRef<HTMLTextAreaElement>(null);
  const [showScrollbar, setShowScrollbar] = React.useState(false);
  const [attachments, setAttachments] = React.useState<ComposerAttachment[]>([]);
  const imageInputRef = React.useRef<HTMLInputElement>(null);

  // Restore draft if provided
  React.useEffect(() => {
    if (draftId) {
      const draft = localStorage.getItem(`draft_${draftId}`);
      if (draft) setText(draft);
    }
  }, [draftId]);

  React.useEffect(() => {
    if (!prefill?.text) return;
    setText(prefill.text);
    requestAnimationFrame(() => {
      const el = textareaRef.current;
      if (!el) return;
      el.focus();
      el.setSelectionRange(el.value.length, el.value.length);
    });
  }, [prefill?.key, prefill?.text]);

  // Save draft
  React.useEffect(() => {
    if (draftId && text) {
      localStorage.setItem(`draft_${draftId}`, text);
    } else if (draftId && !text) {
      localStorage.removeItem(`draft_${draftId}`);
    }
  }, [text, draftId]);

  React.useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;

    const styles = window.getComputedStyle(el);
    const lineHeight = Number.parseFloat(styles.lineHeight) || 20;
    const paddingY =
      (Number.parseFloat(styles.paddingTop) || 0) +
      (Number.parseFloat(styles.paddingBottom) || 0);
    const maxScrollHeight = lineHeight * TEXTAREA_MAX_LINES + paddingY;

    el.style.height = "auto";
    const nextHeight = Math.max(TEXTAREA_MIN_HEIGHT, el.scrollHeight);
    const needsScrollbar = nextHeight > maxScrollHeight;
    el.style.height = `${needsScrollbar ? maxScrollHeight : nextHeight}px`;
    el.style.overflowY = needsScrollbar ? "auto" : "hidden";
    setShowScrollbar(needsScrollbar);
  }, [text, prefill?.key]);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    } else if (e.key === "Escape" && isRunning) {
      // Matches the transcript's "Esc to interrupt" affordance, which the
      // composer used to swallow with a console.log.
      e.preventDefault();
      onStop?.();
    }
  };

  const handleSend = () => {
    // While a run streams there is no send affordance — the button is Stop —
    // so Enter must not quietly drop the text either.
    if ((!text.trim() && attachments.length === 0) || disabled || isRunning) return;
    onSubmit(text, attachments.length > 0 ? attachments : undefined);
    setText("");
    attachments.forEach((a) => {
      if (a.previewUrl) URL.revokeObjectURL(a.previewUrl);
    });
    setAttachments([]);
    if (draftId) localStorage.removeItem(`draft_${draftId}`);
  };

  const handleImageSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (!files?.length) return;

    const next: ComposerAttachment[] = [];
    for (const file of Array.from(files)) {
      if (!file.type.startsWith("image/")) continue;
      const att = await fileToAttachment(file);
      next.push({
        ...att,
        previewUrl: URL.createObjectURL(file),
      });
    }
    if (next.length > 0) {
      setAttachments((prev) => [...prev, ...next]);
    }
    e.target.value = "";
  };

  const removeAttachment = (id: string) => {
    setAttachments((prev) => {
      const removed = prev.find((a) => a.id === id);
      if (removed?.previewUrl) URL.revokeObjectURL(removed.previewUrl);
      return prev.filter((a) => a.id !== id);
    });
  };

  return (
    <div className="composer-wrap">
      <div className="composer-inner">
        <div className="composer">
          {attachments.length > 0 && (
            <div style={{ display: "flex", gap: 8, padding: "8px 12px 0", flexWrap: "wrap" }}>
              {attachments.map((att) => (
                <span key={att.id} className="chip-file" style={{ display: "inline-flex", alignItems: "center", gap: 4 }}>
                  {att.previewUrl ? (
                    <img src={att.previewUrl} alt={att.name} style={{ width: 24, height: 24, objectFit: "cover", borderRadius: 4 }} />
                  ) : (
                    <Icon name="image" size={14} />
                  )}
                  {att.name}
                  <button type="button" className="btn icon sm ghost" onClick={() => removeAttachment(att.id)} aria-label="Remove attachment">
                    <Icon name="x" size={12} />
                  </button>
                </span>
              ))}
            </div>
          )}
          <textarea
            ref={textareaRef}
            className={`input${showScrollbar ? " input-scrollable" : ""}`}
            placeholder={placeholder}
            value={text}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={disabled}
            rows={1}
          />
          <div className="composer-bar">
            <button
              type="button"
              className="cb-icon"
              title="Attach files"
              onClick={() => alert("File attachments coming soon")}
              disabled={disabled}
            >
              <Icon name="clip" size={16} />
            </button>
            <input
              ref={imageInputRef}
              type="file"
              accept="image/*"
              multiple
              style={{ display: "none" }}
              onChange={handleImageSelect}
            />
            <button
              type="button"
              className="cb-icon"
              title="Attach image"
              onClick={() => enableImageAttachments && imageInputRef.current?.click()}
              disabled={disabled || !enableImageAttachments}
            >
              <Icon name="image" size={16} />
            </button>
            <button
              type="button"
              className="cb-icon"
              title="Mention resource (@)"
              onClick={() => setText((prev) => prev + "@")}
              disabled={disabled}
            >
              <Icon name="at" size={16} />
            </button>
            <button
              type="button"
              className="cb-icon"
              title="Slash command (/)"
              onClick={() => setText((prev) => prev + "/")}
              disabled={disabled}
            >
              <Icon name="slash" size={16} />
            </button>
            <span className="spacer" />

            {picker === "mode" ? (
              <ModePicker value={mode} onChange={onModeChange} disabled={disabled} />
            ) : null}

            {useManagedModels ? (
              <ModelPicker
                value={model}
                onChange={onModelChange}
                disabled={disabled}
                effort={picker === "effort" ? effort : undefined}
                onEffortChange={picker === "effort" ? onEffortChange : undefined}
              />
            ) : (
              <span className={`select ${disabled ? "disabled" : ""}`}>{model.label}</span>
            )}

            {isRunning ? (
              <button
                type="button"
                className="send is-stop"
                onClick={onStop}
                aria-label="Stop run"
                title="Stop run (Esc)"
              >
                <Icon name="stop" size={14} />
              </button>
            ) : (
              <button
                type="button"
                className="send"
                onClick={handleSend}
                aria-label="Send prompt"
                disabled={disabled || (!text.trim() && attachments.length === 0)}
              >
                <Icon name="send" size={16} />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
