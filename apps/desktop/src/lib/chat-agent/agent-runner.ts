import { streamChat, type ChatStreamRequest, type ChatToolResultPayload } from '../chat';
import { isModelImageDenial, isUserImageRequest } from './image-intent';
import {
  executeChatImageTool,
  executeTavily,
  parseGeneratedImagePreview,
  searchWeb,
} from './image-tools';
import { parseChatToolCalls, stripChatToolMarkup } from './parse-tool-calls';
import { validateGenerateImageCall } from './tool-guards';
import type {
  ChatAgentStep,
  ChatToolCall,
  ChatToolResult,
  GeneratedImagePreview,
} from './types';

const MAX_AGENT_STEPS = 4;

export type ChatAgentMessage = {
  role: 'user' | 'assistant';
  content: string;
};

export type HomeChatAgentTurnResult = {
  text: string;
  images: GeneratedImagePreview[];
};

export type HomeChatAgentRunParams = {
  request: Omit<ChatStreamRequest, 'text'>;
  text: string;
  priorMessages: ChatAgentMessage[];
  enableImageTool: boolean;
  signal?: AbortSignal;
  onDelta: (text: string) => void;
  onStepsChange: (steps: ChatAgentStep[]) => void;
  onGeneratedImage?: (image: GeneratedImagePreview) => void;
};

function makeStepId(tool: string, index: number): string {
  return `${tool}-${index}-${Date.now()}`;
}

function formatToolResult(name: string, content: string): ChatToolResult {
  return { role: 'tool', name, content };
}

function dedupeToolCalls(calls: ChatToolCall[]): ChatToolCall[] {
  const result: ChatToolCall[] = [];
  let imageIncluded = false;
  for (const call of calls) {
    if (call.name === 'generate_image') {
      if (imageIncluded) continue;
      imageIncluded = true;
    }
    result.push(call);
  }
  return result;
}

async function runGenerateImageTool(
  call: ChatToolCall,
  params: HomeChatAgentRunParams,
  generatedImages: GeneratedImagePreview[],
  runningStep: ChatAgentStep,
  updateSteps: () => void,
): Promise<ChatToolResult> {
  const imagePrompt = String(
    call.arguments.prompt ?? call.arguments.description ?? call.arguments.text ?? '',
  ).trim();
  const validation = validateGenerateImageCall(imagePrompt);
  if (!validation.ok) {
    runningStep.status = 'error';
    updateSteps();
    return formatToolResult(
      call.name,
      JSON.stringify({ error: 'WRONG_TOOL', message: validation.message }),
    );
  }

  const result = await executeChatImageTool(call);
  const preview = parseGeneratedImagePreview(result);
  if (preview) {
    generatedImages.push(preview);
    params.onGeneratedImage?.(preview);
    runningStep.status = 'done';
    updateSteps();
    return formatToolResult(
      call.name,
      JSON.stringify({ ok: true, generated: true, caption: preview.caption ?? null }),
    );
  }

  runningStep.status = 'done';
  updateSteps();
  return formatToolResult(call.name, result);
}

export async function runHomeChatAgentTurn(
  params: HomeChatAgentRunParams,
): Promise<HomeChatAgentTurnResult> {
  const contextMessages = params.priorMessages.map((message) => ({
    role: message.role,
    content: message.content,
  }));

  let toolResults: ChatToolResult[] = [];
  let finalAssistantText = '';
  const steps: ChatAgentStep[] = [];
  const generatedImages: GeneratedImagePreview[] = [];

  const updateSteps = () => {
    params.onStepsChange([...steps]);
  };

  const beginToolStep = (tool: string, label: string): ChatAgentStep => {
    const existing = [...steps]
      .reverse()
      .find((step) => step.tool === tool && (step.status === 'error' || step.status === 'running'));
    if (existing) {
      existing.status = 'running';
      existing.label = label;
      updateSteps();
      return existing;
    }
    const step: ChatAgentStep = {
      id: makeStepId(tool, steps.length),
      tool,
      status: 'running',
      label,
    };
    steps.push(step);
    updateSteps();
    return step;
  };

  const detectStreamingTool = (text: string): string | null => {
    if (
      /(?:\[tool_call\s+name=generate_image|"name"\s*:\s*"generate_image"|```generate_image|<tool_call|<generate_image)/i.test(
        text,
      )
    ) {
      return 'generate_image';
    }
    return /(?:\[tool_call\s+name=(?:web_search|tavily_(?:search|extract|map|crawl|research))|"name"\s*:\s*"(?:web_search|tavily_(?:search|extract|map|crawl|research))"|```(?:web_search|tavily_(?:search|extract|map|crawl|research)))/i.test(
      text,
    )
      ? 'tavily_search'
      : null;
  };

  const ensureStreamingStep = (tool: string) => {
    if (steps.some((step) => step.tool === tool)) return;
    steps.push({
      id: `stream-${tool}`,
      tool,
      status: 'running',
      label: tool === 'generate_image' ? 'Generating image' : 'Searching the web…',
    });
    updateSteps();
  };

  const removeStreamingStep = (tool: string) => {
    const index = steps.findIndex((step) => step.id === `stream-${tool}`);
    if (index === -1) return;
    steps.splice(index, 1);
    updateSteps();
  };

  for (let step = 0; step < MAX_AGENT_STEPS; step += 1) {
    let accumulated = '';

    await streamChat(
      {
        ...params.request,
        text: params.text,
        enableImageTool: params.enableImageTool,
        contextMessages,
        ...(toolResults.length > 0
          ? { toolResults: toolResults as ChatToolResultPayload[] }
          : {}),
      },
      (delta) => {
        accumulated += delta;
        const streamingTool = detectStreamingTool(accumulated);
        if (streamingTool) ensureStreamingStep(streamingTool);
        params.onDelta(stripChatToolMarkup(accumulated));
      },
      params.signal,
    );

    const { assistantText, toolCalls: parsedToolCalls } = parseChatToolCalls(accumulated);
    const toolCalls = dedupeToolCalls(parsedToolCalls);
    finalAssistantText = assistantText;

    if (toolCalls.length === 0) {
      const cleaned = stripChatToolMarkup(finalAssistantText);
      if (
        params.enableImageTool &&
        generatedImages.length === 0 &&
        isUserImageRequest(params.text) &&
        isModelImageDenial(cleaned)
      ) {
        const runningStep = beginToolStep('generate_image', 'Generating image');
        try {
          await runGenerateImageTool(
            { name: 'generate_image', arguments: { prompt: params.text.trim() } },
            params,
            generatedImages,
            runningStep,
            updateSteps,
          );
          if (generatedImages.length > 0) {
            params.onDelta('');
            params.onStepsChange([]);
            return { text: '', images: generatedImages };
          }
        } catch (error) {
          const message = error instanceof Error ? error.message : 'Image generation failed';
          runningStep.status = 'error';
          updateSteps();
          params.onStepsChange([]);
          return { text: message, images: generatedImages };
        }
      }

      params.onDelta(cleaned);
      params.onStepsChange([]);
      return { text: cleaned, images: generatedImages };
    }

    toolResults = [];

    for (const call of toolCalls) {
      removeStreamingStep(call.name);
      const runningStep = beginToolStep(
        call.name,
        call.name === 'generate_image' ? 'Generating image' : call.name,
      );

      try {
        if (call.name === 'web_search' || call.name.startsWith('tavily_')) {
          const query = String(call.arguments.query ?? '').trim();
          if (['web_search', 'tavily_search', 'tavily_research'].includes(call.name) && !query) {
            toolResults.push(
              formatToolResult(call.name, JSON.stringify({ error: 'web_search requires query' })),
            );
            runningStep.status = 'error';
            updateSteps();
            continue;
          }
          toolResults.push(
            formatToolResult(
              call.name,
              JSON.stringify(
                call.name === 'web_search'
                  ? await searchWeb(query)
                  : await executeTavily(call.name, call.arguments),
              ),
            ),
          );
          runningStep.status = 'done';
          updateSteps();
          continue;
        }

        if (call.name !== 'generate_image') {
          toolResults.push(
            formatToolResult(
              call.name,
              JSON.stringify({ error: `Tool "${call.name}" is not available in chat.` }),
            ),
          );
          runningStep.status = 'error';
          updateSteps();
          continue;
        }

        if (!params.enableImageTool) {
          toolResults.push(
            formatToolResult(
              call.name,
              JSON.stringify({ error: 'Image generation is disabled in settings.' }),
            ),
          );
          runningStep.status = 'error';
          updateSteps();
          continue;
        }

        if (generatedImages.length > 0) {
          toolResults.push(
            formatToolResult(call.name, JSON.stringify({ ok: true, alreadyGenerated: true })),
          );
          runningStep.status = 'done';
          updateSteps();
          continue;
        }

        toolResults.push(
          await runGenerateImageTool(call, params, generatedImages, runningStep, updateSteps),
        );
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Tool execution failed';
        toolResults.push(formatToolResult(call.name, JSON.stringify({ error: message })));
        runningStep.status = 'error';
        updateSteps();
      }
    }

    if (generatedImages.length > 0) {
      const cleaned = stripChatToolMarkup(finalAssistantText);
      params.onDelta(cleaned);
      params.onStepsChange([]);
      return { text: cleaned, images: generatedImages };
    }

    params.onDelta('');
  }

  params.onStepsChange([]);
  return { text: stripChatToolMarkup(finalAssistantText), images: generatedImages };
}
