/**
 * Detect provider/API error payloads that upstream sometimes streams as plain text.
 * Never show raw JSON or billing URLs in the chat UI.
 */

const PROVIDER_ERROR_TYPES = new Set([
  'insufficient_quota',
  'api_error',
  'invalid_request_error',
  'authentication_error',
  'permission_error',
  'rate_limit_error',
]);

export function humanizeProviderErrorObject(parsed: Record<string, unknown>): string | null {
  const type = typeof parsed.type === 'string' ? parsed.type : '';
  const code = typeof parsed.code === 'string' ? parsed.code : '';

  if (type === 'insufficient_quota' || code === 'credit_balance_exhausted') {
    return 'API quota exceeded. Add credits in your provider billing settings, then try again.';
  }

  const nested = parsed.error as { message?: string; type?: string } | undefined;
  const nestedType = nested?.type ?? '';
  if (
    PROVIDER_ERROR_TYPES.has(type) ||
    PROVIDER_ERROR_TYPES.has(nestedType) ||
    (type && code)
  ) {
    const raw =
      (typeof parsed.message === 'string' && parsed.message) ||
      nested?.message ||
      null;
    if (raw) return stripTechnicalNoise(raw);
    return 'The model provider returned an error. Please try again.';
  }

  return null;
}

export function humanizeProviderErrorText(text: string): string | null {
  const trimmed = text.trim();
  if (!trimmed) return null;

  if (trimmed.startsWith('{')) {
    try {
      const parsed = JSON.parse(trimmed) as Record<string, unknown>;
      return humanizeProviderErrorObject(parsed);
    } catch {
      // fall through to pattern match
    }
  }

  if (/"\s*type\s*"\s*:\s*"\s*insufficient_quota\s*"/.test(trimmed)) {
    return 'API quota exceeded. Add credits in your provider billing settings, then try again.';
  }

  if (/"\s*type\s*"\s*:\s*"\s*(api_error|invalid_request_error|rate_limit_error)\s*"/.test(trimmed)) {
    return 'The model provider returned an error. Please try again.';
  }

  return null;
}

export function stripTechnicalNoise(message: string): string {
  const cleaned = message
    .replace(/https?:\/\/\S+/g, '')
    .replace(/\s+/g, ' ')
    .trim();

  if (/no credits remaining/i.test(cleaned)) {
    return 'API quota exceeded. Add credits in your provider billing settings, then try again.';
  }
  if (/rate limit/i.test(cleaned)) {
    return 'Rate limit reached. Wait a moment and try again.';
  }
  if (/invalid api key/i.test(cleaned)) {
    return 'API key is invalid or missing. Check your provider configuration.';
  }

  if (cleaned.length > 200) {
    return 'The model provider returned an error. Please try again.';
  }

  return cleaned || 'The model provider returned an error. Please try again.';
}

/** User-facing message for thrown errors (HTTP, stream, provider). */
export function humanizeChatError(error: unknown): string {
  if (error instanceof Error) {
    const fromJson = humanizeProviderErrorText(error.message);
    if (fromJson) return fromJson;
    return stripTechnicalNoise(error.message);
  }
  return 'Something went wrong. Please try again.';
}

export function sanitizeAssistantContent(text: string): {
  content: string;
  isProviderError: boolean;
} {
  const humanized = humanizeProviderErrorText(text);
  if (humanized) {
    return { content: humanized, isProviderError: true };
  }
  return { content: text, isProviderError: false };
}
