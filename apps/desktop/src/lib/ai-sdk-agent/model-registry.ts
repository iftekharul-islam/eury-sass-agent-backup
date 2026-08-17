import { getAgentApiUrl } from '../config';
import { authIpc } from '../auth';
import { createOpenAI } from '@ai-sdk/openai';
import { createAnthropic } from '@ai-sdk/anthropic';
import { createGoogleGenerativeAI } from '@ai-sdk/google';
import { createXai } from '@ai-sdk/xai';
import type { LanguageModel } from 'ai';

function normalizeProvider(provider: string): string {
  const p = provider.trim();
  if (p === 'Claude' || p === 'Anthropic') return 'Claude';
  if (p === 'Google' || p === 'Gemini') return 'Google';
  if (p === 'xAI' || p === 'Grok') return 'xAI';
  return 'OpenAI';
}

export async function resolveLanguageModel(
  provider: string,
  modelId: string,
): Promise<LanguageModel> {
  const tokens = await authIpc.getTokens();
  const base = `${getAgentApiUrl()}/agent/v1/proxy`;
  const internal = normalizeProvider(provider);

  switch (internal) {
    case 'Claude':
      return createAnthropic({
        apiKey: tokens.access_token,
        baseURL: `${base}/anthropic/v1`,
      }).chat(modelId);
    case 'Google':
      return createGoogleGenerativeAI({
        apiKey: tokens.access_token,
        baseURL: `${base}/google/v1beta`,
      }).chat(modelId);
    case 'xAI':
      return createXai({
        apiKey: tokens.access_token,
        baseURL: `${base}/xai/v1`,
      }).chat(modelId);
    case 'OpenAI':
    default:
      return createOpenAI({
        apiKey: tokens.access_token,
        baseURL: `${base}/openai/v1`,
      }).chat(modelId);
  }
}
