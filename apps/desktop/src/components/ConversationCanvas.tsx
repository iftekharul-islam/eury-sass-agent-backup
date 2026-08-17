import * as React from "react";
import { Icon } from "./Icons";
import { ChatTurn, ChatTurnHoverMeta } from "./chat/ChatTurn";
import { ThinkingBlock } from "./chat/ThinkingBlock";
import { ToolGroup } from "./tools/ToolGroup";
import { ApprovalCard } from "./tools/ApprovalCard";
import { MarkdownRenderer } from "./MarkdownRenderer";
import { RunStatus } from "./RunStatus";
import { stripToolCallFences } from "../lib/assistant-text";
import { RunCommandProvider } from "../lib/run-command";
import { useStickToBottom } from "../lib/use-stick-to-bottom";
import { PreviewCard } from "./PreviewCard";
import { selectActivePreview, usePreviewServers } from "../lib/preview-servers";
import { COMPOSER_MODES } from "./ModePicker";
import { Composer } from "./Composer";
import { useAppSettings } from "../lib/settings";
import { startManagedRun, effectiveRunMode } from "../lib/run";
import { ipcClient } from "../lib/ipc";
import {
  isWriteTool,
  selectActiveRun,
  selectHistory,
  selectRunActivities,
  selectTranscript,
  sessionStore,
  useSessionState,
  type RunRecord,
  type SessionState,
  type ToolActivityRecord,
  type TranscriptTurn,
} from "../lib/session-store";
import type { ComposerAttachment } from "./Composer";

export interface ConversationCanvasProps {
  onScreenChange?: (screen: "run" | "approve-cmd" | "approve-diff" | "plan" | "changes" | "runs") => void;
  onSubmitMessage?: (text: string) => void;
  conversationId?: string;
  workspaceRoot?: string;
  workspaceName?: string;
  /** Run mode, owned by the app so the title bar and composer agree. */
  mode?: string;
  onModeChange?: (mode: string) => void;
  /** Opens the folder picker; shown when no project is attached. */
  onOpenProject?: () => void;
  /** False while the open project is untrusted — Eury is read-only there. */
  isWorkspaceTrusted?: boolean;
  onTrustWorkspace?: () => void;
  /** A file path pulled from the file tree, to seed into the composer. */
  mention?: string | null;
  onMentionConsumed?: () => void;
  /**
   * Runs a shell snippet from the transcript in the workspace terminal. Absent
   * when there is no terminal to run it in, which hides the Run affordance.
   */
  onRunCommand?: (command: string) => void;
}

function formatTimestamp(at: number): string {
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(new Date(at));
}

function visibleTurnText(turn: TranscriptTurn): string {
  return stripToolCallFences(turnText(turn)).trim();
}

function turnText(turn: TranscriptTurn): string {
  return turn.blocks
    .filter((block): block is { kind: "text"; text: string } => block.kind === "text")
    .map((block) => block.text)
    .join("");
}

/**
 * The turn's blocks, with consecutive tool calls collected into one group so
 * they can be summarized ("Ran a command, read 2 files") instead of stacking.
 */
type RenderBlock =
  | { kind: "text"; key: string; text: string }
  | { kind: "tools"; key: string; activities: ToolActivityRecord[] };

function groupBlocks(
  turn: TranscriptTurn,
  activities: SessionState["activities"],
): RenderBlock[] {
  const rendered: RenderBlock[] = [];

  turn.blocks.forEach((block, index) => {
    if (block.kind === "text") {
      // The tool card below shows the call; the raw fence is noise.
      const visible = stripToolCallFences(block.text);
      if (!visible) return;
      rendered.push({ kind: "text", key: `text-${index}`, text: visible });
      return;
    }

    const activity = activities[block.toolCallId];
    if (!activity) return;
    // A call still waiting on the user renders as its own approval prompt.
    if (activity.status === "pending_approval") return;

    const last = rendered[rendered.length - 1];
    if (last?.kind === "tools") {
      last.activities.push(activity);
      return;
    }
    rendered.push({ kind: "tools", key: `tools-${index}`, activities: [activity] });
  });

  return rendered;
}

/**
 * An assistant turn in the Home chat's shape — plain prose, no avatar rail —
 * with this area's extra content (reasoning, tool cards, approvals) folded
 * into the same column.
 */
function AssistantTurn({
  turn,
  run,
  activities,
  onApprove,
  onViewDiff,
  time,
  modelName,
  copyText,
  onRetry,
  actionsDisabled = false,
}: {
  turn: TranscriptTurn;
  run?: RunRecord;
  activities: SessionState["activities"];
  onApprove: (toolCallId: string, decision: "allow_once" | "allow_session" | "denied") => void;
  onViewDiff: () => void;
  time?: string;
  modelName?: string;
  copyText?: string;
  onRetry?: () => void;
  actionsDisabled?: boolean;
}) {
  const thinking = turn.thinking?.trim() ?? "";
  const isRunning = run?.status === "running";
  const runActivities = run?.activityIds.map((id) => activities[id]).filter(Boolean) ?? [];
  const runningTool = runActivities.find((activity) => activity?.status === "running")?.name;
  const rendered = groupBlocks(turn, activities);
  // Calls this turn is blocked on, shown where they happened rather than in a
  // dock at the far end of the window.
  const pending = turn.blocks
    .filter((block): block is { kind: "tool"; toolCallId: string } => block.kind === "tool")
    .map((block) => activities[block.toolCallId])
    .filter((activity): activity is ToolActivityRecord => activity?.status === "pending_approval");

  const hasVisibleOutput = rendered.length > 0 || pending.length > 0;
  // Keep "Thinking" up until the first answer text or tool card lands — not
  // only while `thinkingStartedAt` is set (that clears when text arrives).
  const thinkingLive = Boolean(isRunning && !hasVisibleOutput);
  const showThinking = Boolean(
    thinking || thinkingLive || (!isRunning && turn.thinkingMs && turn.thinkingMs > 0),
  );
  const showRunStatus = Boolean(
    isRunning &&
      run &&
      !thinkingLive &&
      (hasVisibleOutput || run.phase === "tool" || run.phase === "awaiting_approval"),
  );
  const isEmpty = turn.blocks.length === 0 && !showThinking;

  return (
    <div className="chat-turn-assistant-wrap">
      <div className="turn turn-assistant">
        <div className="turn-body">
          {showThinking && (
            <ThinkingBlock
              text={thinking}
              live={thinkingLive}
              durationMs={turn.thinkingMs}
            />
          )}

          {rendered.map((block) =>
            block.kind === "text" ? (
              <MarkdownRenderer key={block.key} content={block.text} />
            ) : (
              <ToolGroup
                key={block.key}
                activities={block.activities}
                live={Boolean(isRunning)}
                onViewDiff={onViewDiff}
              />
            ),
          )}

          {showRunStatus && run && (
            <RunStatus
              run={run}
              thinking={thinking}
              toolName={runningTool ?? pending[0]?.name}
            />
          )}

          {pending.map((activity) => (
            <ApprovalCard
              key={activity.id}
              activity={activity}
              isWrite={isWriteTool(activity.name)}
              onAllowOnce={() => onApprove(activity.id, "allow_once")}
              onAllowSession={() => onApprove(activity.id, "allow_session")}
              onDeny={() => onApprove(activity.id, "denied")}
            />
          ))}

          {/* Every turn ends somewhere visible: a failed or stopped run says
              so here rather than leaving a silent gap in the transcript. */}
          {run?.status === "failed" && (
            <div className="chat-error-banner">{run.error ?? "This run failed."}</div>
          )}
          {run?.status === "cancelled" && (
            <div className="chat-turn-note">Stopped{isEmpty ? " before Eury replied" : ""}.</div>
          )}
          {run?.status === "completed" && isEmpty && (
            <div className="chat-turn-note">Eury finished without sending a reply.</div>
          )}
        </div>
      </div>
      {copyText ? (
        <ChatTurnHoverMeta
          modelName={modelName}
          time={time}
          copyText={copyText}
          onRetry={onRetry}
          align="assistant"
          actionsDisabled={actionsDisabled}
        />
      ) : null}
    </div>
  );
}

export function ConversationCanvas({
  onScreenChange,
  onSubmitMessage,
  conversationId = "default-conversation",
  workspaceRoot,
  workspaceName,
  mode,
  onModeChange,
  onOpenProject,
  isWorkspaceTrusted = true,
  onTrustWorkspace,
  mention,
  onMentionConsumed,
  onRunCommand,
}: ConversationCanvasProps) {
  const [settings, store] = useAppSettings();
  const session = useSessionState();
  // Anything this session started serving, so "run it" ends in the running app
  // rather than in a URL the user has to carry elsewhere.
  const preview = selectActivePreview(usePreviewServers());

  // Seeds a file path from the tree into the composer. `key` forces the
  // Composer to re-apply even when the same path is picked twice.
  const [composerPrefill, setComposerPrefill] = React.useState<{
    key: number;
    text: string;
  } | null>(null);

  React.useEffect(() => {
    if (!mention) return;
    setComposerPrefill((prev) => ({ key: (prev?.key ?? 0) + 1, text: `@${mention} ` }));
    onMentionConsumed?.();
  }, [mention, onMentionConsumed]);

  // Falls back to local state when the app does not own the mode.
  const [localMode, setLocalMode] = React.useState("Agent");
  const composerMode = mode ?? localMode;
  const setComposerMode = onModeChange ?? setLocalMode;
  const [startError, setStartError] = React.useState<string | null>(null);
  const composerModel = {
    provider: settings.model.activeProvider,
    modelId: settings.model.activeModelId,
    label: settings.model.activeModelLabel,
  };

  const handleModelChange = (model: { provider: string; modelId: string; label: string }) => {
    store.updateModel({
      activeProvider: model.provider,
      activeModelId: model.modelId,
      activeModelLabel: model.label,
    });
  };

  // A read-only mode is the usual reason a change request comes back refused,
  // so the composer says which mode is active rather than letting the model
  // explain it after the fact.
  const modeCanEdit =
    COMPOSER_MODES.find((option) => option.id === composerMode)?.canEdit ?? true;

  const turns = selectTranscript(session, conversationId);
  const activeRun = selectActiveRun(session, conversationId);
  const pendingApprovals = activeRun
    ? selectRunActivities(session, activeRun.id).filter(
        (activity) => activity.status === "pending_approval",
      )
    : [];
  const runsById = React.useMemo(
    () => new Map(session.runs.map((run) => [run.id, run])),
    [session.runs],
  );

  const {
    ref: scrollRef,
    onScroll: handleScroll,
    onWheel: handleWheel,
    jumpToBottom,
  } = useStickToBottom<HTMLDivElement>();

  // An approval blocks the run, so it is the one thing worth pulling the user
  // back down for even when they have scrolled away.
  const pendingIds = pendingApprovals.map((activity) => activity.id).join(",");
  React.useEffect(() => {
    if (!pendingIds) return;
    jumpToBottom();
  }, [pendingIds, jumpToBottom]);

  React.useEffect(() => {
    setStartError(null);
  }, [conversationId]);

  const interruptRun = React.useCallback(() => {
    sessionStore.cancelActiveRun(conversationId);
    void ipcClient.run.cancel(conversationId).catch((err) => {
      console.error("Failed to cancel run:", err);
    });
  }, [conversationId]);

  // The streaming bar promises "Esc to interrupt", so Escape has to actually
  // cancel the run while one is in flight.
  React.useEffect(() => {
    if (!activeRun) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      e.preventDefault();
      interruptRun();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [activeRun?.id, interruptRun]);

  const handleApprove = (
    toolCallId: string,
    decision: "allow_once" | "allow_session" | "denied",
  ) => {
    sessionStore.recordApprovalDecision(toolCallId, decision);
    void ipcClient.run.approve(toolCallId, decision).catch((err) => {
      console.error("Failed to send approval decision:", err);
    });
  };

  const handleSend = async (text: string, attachments?: ComposerAttachment[]) => {
    const trimmed = text.trim();
    if (!trimmed && !attachments?.length) return;
    // Read the store, not the render's snapshot: two quick sends both close
    // over `activeRun === undefined` and the core rejects the second run.
    if (selectActiveRun(sessionStore.getState(), conversationId)) return;

    if (workspaceRoot && !isWorkspaceTrusted) {
      setStartError(
        "This project is not trusted yet — Eury can only read files until you click Trust project.",
      );
      return;
    }

    const prompt = trimmed || "Describe the attached image(s).";
    setStartError(null);

    // Keep the Rust workspace path in sync with every run so trust, terminals,
    // and the sandbox all target the folder the UI is showing.
    if (workspaceRoot) {
      try {
        await ipcClient.workspace.open(workspaceRoot);
      } catch (err) {
        const msg = err instanceof Error ? err.message : "Failed to open workspace";
        setStartError(msg);
        return;
      }
    }

    // Captured before this turn is appended, so the model gets the prior
    // conversation and not a copy of the message it is answering.
    const history = selectHistory(sessionStore.getState(), conversationId);

    sessionStore.appendUserTurn({
      conversationId,
      who: settings.profile.preferredName || "You",
      avatar: settings.profile.avatar || settings.profile.preferredName[0] || "U",
      text: trimmed,
      modeBadge: composerMode,
      fileChip: attachments?.length
        ? `${attachments.length} image${attachments.length > 1 ? "s" : ""}`
        : undefined,
    });
    onSubmitMessage?.(trimmed);

    const runId = crypto.randomUUID();
    sessionStore.startRun({
      runId,
      conversationId,
      prompt,
      mode: composerMode,
      modelLabel: settings.model.activeModelLabel,
      workspaceRoot,
      workspaceName,
    });

    try {
      await startManagedRun({
        runId,
        conversationId,
        mode: effectiveRunMode(composerMode, workspaceRoot, prompt),
        prompt,
        history,
        provider: settings.model.activeProvider,
        modelId: settings.model.activeModelId,
        workspaceRoot,
        attachments: attachments?.map(({ id, name, contentType, dataBase64 }) => ({
          id,
          name,
          contentType,
          dataBase64,
        })),
      });
    } catch (err) {
      const msg =
        err instanceof Error
          ? err.message
          : typeof err === "string"
            ? err
            : "Failed to start run";
      sessionStore.failRun(runId, msg);
      setStartError(msg);
      console.error("Failed to start managed run:", err);
    }
  };

  return (
    <RunCommandProvider onRunCommand={onRunCommand ?? null}>
    <div className="chat-surface code-chat-surface">
      <div className="convo" ref={scrollRef} onScroll={handleScroll} onWheel={handleWheel}>
        <div className="convo-col">
          {turns.length === 0 && (
            <div className="chat-empty-hint">
              {workspaceRoot
                ? `Ask Eury to work on ${workspaceName ?? "this project"}.`
                : "Open a project folder to give Eury a workspace to act in."}
            </div>
          )}

          {turns.map((turn, index) => {
            if (turn.sender === "user") {
              return (
                <ChatTurn
                  key={turn.id}
                  id={turn.id}
                  variant="chat"
                  sender="user"
                  who={turn.who}
                  time={formatTimestamp(turn.at)}
                  avatar={turn.avatar}
                  text={turnText(turn)}
                  fileChip={turn.fileChip}
                  showActions={false}
                  actionsDisabled={!!activeRun}
                  onEdit={() =>
                    setComposerPrefill((prev) => ({
                      key: (prev?.key ?? 0) + 1,
                      text: turnText(turn),
                    }))
                  }
                  onRetry={() => void handleSend(turnText(turn))}
                />
              );
            }

            const run = turn.runId ? runsById.get(turn.runId) : undefined;
            const precedingUser = [...turns.slice(0, index)]
              .reverse()
              .find((item) => item.sender === "user");

            return (
              <AssistantTurn
                key={turn.id}
                turn={turn}
                run={run}
                activities={session.activities}
                onApprove={handleApprove}
                onViewDiff={() => onScreenChange?.("changes")}
                time={formatTimestamp(turn.at)}
                modelName={run?.modelLabel}
                copyText={visibleTurnText(turn) || undefined}
                actionsDisabled={!!activeRun}
                onRetry={
                  precedingUser
                    ? () => void handleSend(turnText(precedingUser))
                    : undefined
                }
              />
            );
          })}

          {startError && <div className="chat-error-banner">{startError}</div>}

          {preview && <PreviewCard server={preview} />}
        </div>

      </div>

      {!workspaceRoot && (
        <div className="workspace-notice">
          <Icon name="folder" size={14} />
          <span>No project open — Eury can't read or change files.</span>
          {onOpenProject && (
            <button type="button" className="btn sm" onClick={onOpenProject}>
              Open project
            </button>
          )}
        </div>
      )}

      {workspaceRoot && !isWorkspaceTrusted && (
        <div className="workspace-notice is-untrusted">
          <Icon name="shield" size={14} />
          <span>
            {workspaceName ?? "This project"} is untrusted — Eury can read it, but shell
            commands and file writes are blocked until you trust it.
          </span>
          {onTrustWorkspace && (
            <button type="button" className="btn sm primary" onClick={onTrustWorkspace}>
              Trust project
            </button>
          )}
        </div>
      )}

      <Composer
        mode={composerMode}
        model={composerModel}
        prefill={composerPrefill}
        onModeChange={setComposerMode}
        onModelChange={handleModelChange}
        onSubmit={handleSend}
        isRunning={!!activeRun}
        onStop={interruptRun}
        enableImageAttachments={settings.capabilities.enableImageInspection}
        placeholder={
          !workspaceRoot
            ? "Write a message…"
            : modeCanEdit
            ? "Ask Eury to change this project…"
            : `Ask about this project — ${composerMode} mode can't change files`
        }
      />
      <p className="chat-disclaimer">
        Eury is AI and can make mistakes. Please double-check responses.
      </p>
    </div>
    </RunCommandProvider>
  );
}
