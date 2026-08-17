import { invoke } from '@tauri-apps/api/core';
import { tool, type ToolSet } from 'ai';
import { z } from 'zod';

type ToolMode = 'chat' | 'ask' | 'plan' | 'agent' | 'build';

const READ_TOOLS = new Set(['read_file', 'list_dir', 'glob', 'grep']);
const WRITE_TOOLS = new Set(['write_file', 'edit_file']);
const EXECUTE_TOOLS = new Set(['run_command']);

function modeAllowsTool(mode: ToolMode, name: string): boolean {
  if (READ_TOOLS.has(name)) {
    return mode !== 'chat';
  }
  if (WRITE_TOOLS.has(name)) {
    return mode === 'agent' || mode === 'build';
  }
  if (EXECUTE_TOOLS.has(name)) {
    return mode === 'agent' || mode === 'build' || mode === 'ask';
  }
  return false;
}

export type LocalToolContext = {
  runId: string;
  conversationId: string;
  mode: ToolMode;
  workspaceRoot?: string;
};

async function executeTool(
  ctx: LocalToolContext,
  toolCallId: string,
  name: string,
  argumentsJson: Record<string, unknown>,
): Promise<unknown> {
  return invoke<unknown>('tool_execute', {
    request: {
      run_id: ctx.runId,
      conversation_id: ctx.conversationId,
      tool_call_id: toolCallId,
      name,
      arguments: argumentsJson,
      mode: ctx.mode,
      workspace_root: ctx.workspaceRoot ?? null,
    },
  });
}

export function buildLocalTools(ctx: LocalToolContext): ToolSet {
  const tools: ToolSet = {};

  const maybeAdd = (
    name: string,
    description: string,
    schema: z.ZodObject<z.ZodRawShape>,
  ) => {
    if (!modeAllowsTool(ctx.mode, name)) return;
    tools[name] = tool({
      description,
      inputSchema: schema,
      execute: async (args, { toolCallId }) => {
        return executeTool(
          ctx,
          toolCallId,
          name,
          args as Record<string, unknown>,
        );
      },
    });
  };

  maybeAdd(
    'read_file',
    'Read a text file relative to the workspace root.',
    z.object({ path: z.string().describe('Relative file path') }),
  );
  maybeAdd(
    'list_dir',
    'List entries in a directory relative to the workspace root.',
    z.object({ path: z.string().describe('Relative directory path') }),
  );
  maybeAdd(
    'glob',
    'Find files matching a glob pattern under the workspace.',
    z.object({ pattern: z.string().describe('Glob pattern') }),
  );
  maybeAdd(
    'grep',
    'Search file contents under the workspace.',
    z.object({
      pattern: z.string().describe('Regex or text pattern'),
      path: z.string().optional().describe('Optional subdirectory'),
    }),
  );
  maybeAdd(
    'write_file',
    'Create or overwrite a file relative to the workspace root.',
    z.object({
      path: z.string(),
      content: z.string(),
    }),
  );
  maybeAdd(
    'edit_file',
    'Replace text inside an existing file.',
    z.object({
      path: z.string(),
      old_text: z.string(),
      new_text: z.string(),
    }),
  );
  maybeAdd(
    'run_command',
    'Run a shell command in the workspace root.',
    z.object({
      command: z.string().describe('Shell command to run'),
    }),
  );

  return tools;
}
