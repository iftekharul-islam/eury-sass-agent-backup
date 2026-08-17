export type ChatToolCall = {
  name: string;
  arguments: Record<string, unknown>;
};

export type ChatToolResult = {
  role: 'tool';
  name: string;
  content: string;
};

export type GeneratedImagePreview = {
  dataUrl: string;
  caption?: string;
  mimeType?: string;
};

export type ChatAgentStep = {
  id: string;
  tool: string;
  status: 'running' | 'done' | 'error';
  label: string;
};
