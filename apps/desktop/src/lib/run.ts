import { ipcClient, type HistoryMessage, type RunRequest } from './ipc';
import { runAiSdkAgentTurn } from './ai-sdk-agent/agent-runner';

export interface AttachmentPayload {
  id: string;
  name: string;
  contentType: string;
  dataBase64: string;
}

export interface StartRunOptions {
  /** Caller-supplied id so the UI can track the run before it starts. */
  runId?: string;
  conversationId: string;
  mode: 'chat' | 'ask' | 'plan' | 'agent' | 'build';
  prompt: string;
  /** Prior turns, oldest first. Sent so the model understands follow-ups. */
  history?: HistoryMessage[];
  provider: string;
  modelId: string;
  attachments?: AttachmentPayload[];
  workspaceRoot?: string;
  isTrusted?: boolean;
  temperature?: number;
  maxTokens?: number;
}

const MODE_MAP: Record<StartRunOptions['mode'], RunRequest['mode']> = {
  chat: 'chat',
  ask: 'ask',
  plan: 'plan',
  agent: 'agent',
  build: 'build',
};

/** User wants files created or the project scaffolded — needs Agent, not Ask/Plan. */
const WRITE_INTENT =
  /\b(create|scaffold|implement|generate|make|build|add|write|setup|set up|init|bootstrap)\b/i;

export async function startManagedRun(options: StartRunOptions): Promise<string> {
  const runId = options.runId ?? crypto.randomUUID();
  const modeKey = options.mode.toLowerCase() as StartRunOptions['mode'];

  const useAiSdk =
    options.workspaceRoot &&
    (modeKey === 'agent' || modeKey === 'build' || modeKey === 'plan' || modeKey === 'ask');

  if (useAiSdk) {
    const controller = new AbortController();
    activeAiSdkControllers.set(options.conversationId, controller);
    try {
      await runAiSdkAgentTurn({
        runId,
        conversationId: options.conversationId,
        mode: modeKey,
        prompt: options.prompt,
        history: options.history,
        provider: options.provider,
        modelId: options.modelId,
        workspaceRoot: options.workspaceRoot,
        isTrusted: options.isTrusted,
        signal: controller.signal,
      });
    } finally {
      activeAiSdkControllers.delete(options.conversationId);
    }
    return runId;
  }

  await ipcClient.run.start({
    runId,
    conversationId: options.conversationId,
    mode: MODE_MAP[modeKey] ?? 'agent',
    prompt: options.prompt,
    history: options.history,
    attachments: options.attachments,
    workspaceRoot: options.workspaceRoot,
    model: {
      provider: options.provider,
      model: options.modelId,
      temperature: options.temperature ?? 0.7,
      maxTokens: options.maxTokens,
    },
  });

  return runId;
}

const activeAiSdkControllers = new Map<string, AbortController>();

/** Cancels an in-flight AI SDK run for a conversation. */
export function cancelAiSdkRun(conversationId: string) {
  const controller = activeAiSdkControllers.get(conversationId);
  if (controller) controller.abort();
}

export function composerModeToRunMode(mode: string): StartRunOptions['mode'] {
  const normalized = mode.toLowerCase();
  if (normalized === 'chat') return 'chat';
  if (normalized === 'ask') return 'ask';
  if (normalized === 'plan') return 'plan';
  if (normalized === 'build') return 'build';
  return 'agent';
}

/** Maps the composer mode to the run mode the core should use. */
export function effectiveRunMode(
  mode: string,
  workspaceRoot?: string,
  prompt?: string,
): StartRunOptions['mode'] {
  const mapped = composerModeToRunMode(mode);
  if (!workspaceRoot) return mapped;
  // Chat in the code area is conversation-only on the wire, but with a project
  // open the user still expects read + diagnostic shell tools.
  if (mapped === 'chat') return 'ask';
  // Creating or editing files needs Agent even if the picker still says Ask/Plan.
  if (
    prompt &&
    WRITE_INTENT.test(prompt) &&
    (mapped === 'ask' || mapped === 'plan')
  ) {
    return 'agent';
  }
  return mapped;
}
