/** Matches `agent_sandbox::pty::MAX_SESSIONS_PER_WORKSPACE`. Kept as a
 * literal here since the frontend has no way to import a Rust constant;
 * `terminal_create` is the source of truth and rejects the 5th session
 * regardless of what this says — this only drives the "+" button's
 * disabled state so the user doesn't fire a request that's certain to
 * fail. */
export const MAX_TERMINAL_SESSIONS = 4;
