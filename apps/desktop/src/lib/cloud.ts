import { authIpc, handleUnauthorized } from './auth';
import { getAgentApiUrl } from './config';

const API_BASE = `${getAgentApiUrl()}/agent/v1`;

async function agentFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const tokens = await authIpc.getTokens();
  const response = await fetch(`${API_BASE}${path}`, {
    ...init,
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${tokens.access_token}`,
      ...(init?.headers ?? {}),
    },
  });

  if (response.status === 401) {
    await handleUnauthorized();
    throw new Error('Session expired — please sign in again.');
  }

  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    const message =
      (body as { error?: { message?: string } })?.error?.message ??
      `Request failed (${response.status})`;
    throw new Error(message);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return response.json() as Promise<T>;
}

export interface AgentModelCatalogItem {
  provider: string;
  id: string;
  label: string;
  contextWindow: number;
  maxOutputTokens: number;
  supportsTools: boolean;
  supportsVision: boolean;
  supportsImageGeneration: boolean;
  maxImageBytes: number;
  maxImageCount: number;
  tier: string;
  allowed: boolean;
  disabledReason: string | null;
  promptTokenMicros: number;
  completionTokenMicros: number;
}

export interface AgentModelsResponse {
  models: AgentModelCatalogItem[];
  default: { provider: string; id: string };
  pricingVersion: string;
  etag: string;
}

export interface AgentUsageResponse {
  limits: {
    dailyManagedRuns: { used: number; limit: number; period: string; resetsAt: string };
    monthlyManagedTokens: { used: number; limit: number; period: string; resetsAt: string };
    monthlyBudgetUsdMicros: { used: number; limit: number; period: string; resetsAt: string };
  };
  concurrentRuns: { used: number; limit: number };
  throttled: boolean;
  estimatesAreApproximate?: boolean;
}

export interface AgentMeResponse {
  userId: string;
  deviceId: string;
  scopes: string[];
  permissions: string[];
  limits: { maxDailyRuns: number };
  user: { id: string; email: string; name: string; plan: string } | null;
}

export const cloudApi = {
  getModels: () => agentFetch<AgentModelsResponse>('/models'),
  getUsage: () => agentFetch<AgentUsageResponse>('/usage/current'),
  getMe: () => agentFetch<AgentMeResponse>('/auth/me'),
};

export function estimateRunCostMicros(
  model: AgentModelCatalogItem,
  promptTokens: number,
  completionTokens: number,
): number {
  return (
    promptTokens * model.promptTokenMicros +
    completionTokens * model.completionTokenMicros
  );
}

export function formatMicrosUsd(micros: number): string {
  return `$${(micros / 1_000_000).toFixed(4)}`;
}

export async function fileToAttachment(file: File): Promise<{
  id: string;
  name: string;
  contentType: string;
  dataBase64: string;
}> {
  const buffer = await file.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]!);
  }
  return {
    id: crypto.randomUUID(),
    name: file.name,
    contentType: file.type || 'image/png',
    dataBase64: btoa(binary),
  };
}
