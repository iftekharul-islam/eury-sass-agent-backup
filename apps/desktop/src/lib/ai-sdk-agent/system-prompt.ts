export type TaskIntent =
  | 'scaffold_project'
  | 'run_dev_server'
  | 'fix_error'
  | 'explain'
  | 'other';

const SCAFFOLD_SIGNALS =
  /\b(create|scaffold|implement|generate|make|build|add|write|setup|set up|init|bootstrap|banate|banao|banaw)\b/i;
const RUN_SIGNALS =
  /\b(run|start|launch|chala|chalaw|locally|dev server|dev)\b/i;
const FIX_SIGNALS = /\b(fix|error|broken|fail|thik|repair|debug)\b/i;
const STACK_SIGNALS = /\b(next\.?js|react|vue|tailwind|pnpm|npm|node)\b/i;

export function detectTaskIntent(prompt: string): TaskIntent {
  const text = prompt.trim();
  if (!text) return 'other';
  if (FIX_SIGNALS.test(text)) return 'fix_error';
  if (RUN_SIGNALS.test(text) && /\b(app|server|dev|locally)\b/i.test(text)) {
    return 'run_dev_server';
  }
  if (SCAFFOLD_SIGNALS.test(text) || STACK_SIGNALS.test(text)) {
    return 'scaffold_project';
  }
  if (/\b(explain|what|how|why|show|list|describe)\b/i.test(text)) {
    return 'explain';
  }
  return 'other';
}

function successCriteria(intent: TaskIntent): string {
  switch (intent) {
    case 'scaffold_project':
      return 'Required files exist, dependencies are installed, and the project can start.';
    case 'run_dev_server':
      return 'A dev server is running and reachable (localhost URL in command output).';
    case 'fix_error':
      return 'The reported error is resolved and verify commands pass.';
    case 'explain':
      return 'The user question is answered clearly using tool results.';
    default:
      return 'The user request is fully addressed.';
  }
}

export function buildTaskEnvelope(prompt: string): string {
  const intent = detectTaskIntent(prompt);
  return [
    '## Active task',
    `Intent: ${intent}`,
    `Request: "${prompt.trim()}"`,
    `Done when: ${successCriteria(intent)}`,
  ].join('\n');
}

export type SystemPromptInput = {
  mode: string;
  workspaceRoot?: string;
  isTrusted?: boolean;
  hasTools: boolean;
};

export function buildSystemPrompt(input: SystemPromptInput): string {
  const lines: string[] = [
    'You are Eury Agent, a coding assistant running on the user machine inside the Eury desktop app.',
    `Current mode: ${input.mode}.`,
    '',
  ];

  if (input.workspaceRoot) {
    lines.push('## Workspace', `Root: ${input.workspaceRoot}`);
    if (input.isTrusted) {
      lines.push(
        'Trust: trusted — you may read files, write files, and run commands. Writes and commands may pause for approval.',
      );
    } else {
      lines.push(
        'Trust: UNTRUSTED — read-only until the user trusts this project.',
      );
    }
    lines.push('All tool paths are relative to this root.');
    lines.push('');
  } else {
    lines.push(
      '## Workspace',
      'No project folder is open — you cannot read, write, or run anything on disk.',
      '',
    );
  }

  if (input.hasTools) {
    lines.push(
      '## How to work',
      '- Use tools to inspect and change the workspace — never ask the user to paste paths or run commands manually.',
      '- To list directories, prefer `list_dir` (not `run_command` with find/ls) unless you need shell output.',
      '- After a command fails, read the output, fix the root cause, and retry.',
      '- For action tasks, keep using tools until the job is actually done — not after one read or plan.',
      '- Verify with build, test, or run commands when available.',
      '- Be concise: report what you did and what happened, including failures.',
      '',
    );
  }

  return lines.join('\n');
}

export function wrapUserMessage(prompt: string): string {
  return `${buildTaskEnvelope(prompt)}\n\n${prompt.trim()}`;
}
