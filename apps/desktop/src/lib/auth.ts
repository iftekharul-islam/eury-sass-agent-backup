import { invoke } from '@tauri-apps/api/core';

export interface AuthTokens {
  access_token: string;
  refresh_token: string;
  /** Required by the platform's refresh endpoint. */
  device_id?: string;
  /** Unix seconds; the core refreshes shortly before this. */
  expires_at?: number;
}

/** Fired when the platform rejects the session and it has been cleared. */
export const SESSION_EXPIRED_EVENT = 'eury:session-expired';

let cachedTokens: AuthTokens | null = null;

export const authIpc = {
  getTokens: async (): Promise<AuthTokens> => {
    // Not cached here: the Rust side refreshes on this call, and a stale copy
    // would hand callers a token the platform has already stopped accepting.
    cachedTokens = await invoke<AuthTokens>('agent_auth_get_tokens');
    return cachedTokens;
  },
  setTokens: async (tokens: AuthTokens): Promise<void> => {
    cachedTokens = tokens;
    await invoke<void>('agent_auth_set_tokens', { tokens });
  },
  clearTokens: async (): Promise<void> => {
    cachedTokens = null;
    await invoke<void>('agent_auth_clear_tokens');
  },
};

/**
 * Drops a session the platform has rejected and tells the app to show sign-in.
 *
 * A 401 used to leave the desktop believing it was signed in while every
 * request failed, with no way back to the sign-in screen short of a restart.
 */
export async function handleUnauthorized(): Promise<void> {
  cachedTokens = null;
  try {
    await authIpc.clearTokens();
  } catch {
    // Even if the store write fails, the UI must still fall back to sign-in.
  }
  window.dispatchEvent(new CustomEvent(SESSION_EXPIRED_EVENT));
}

export function generateCodeVerifier() {
  const array = new Uint32Array(56 / 2);
  crypto.getRandomValues(array);
  return Array.from(array, dec => ('0' + dec.toString(16)).substr(-2)).join('');
}

export async function generateCodeChallenge(verifier: string) {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const hash = await crypto.subtle.digest('SHA-256', data);
  return btoa(String.fromCharCode(...new Uint8Array(hash)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
}
