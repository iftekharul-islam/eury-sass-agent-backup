# Offline and Degraded Modes

Spec-Version: 1.0.0

## Definitions

| Mode | Cloud | LLM | Capabilities |
|------|-------|-----|--------------|
| **Online** | Reachable | Available | Full |
| **Offline** | Unreachable | N/A | Local editor, history, no new agent runs |
| **Degraded cloud** | Reachable, gateway errors | Partial | BYOK fallback if configured |
| **Degraded model** | OK | Rate limited / timeout | Retry, queue, suggest smaller model |
| **Offline grace** | Unreachable | Cached session | Read-only agent history; no sync |

## Offline behavior

When cloud is unreachable:

| Feature | Behavior |
|---------|----------|
| Sign in | Blocked (unless valid refresh token + offline grace window — 24 h) |
| Agent run (managed) | Blocked with clear message |
| Agent run (BYOK) | Works if provider reachable |
| Editor / file tree | Full local access |
| Conversation history | Local SQLite |
| Settings | Local read/write |
| Update check | Skipped |

UI MUST show persistent offline indicator in status bar.

## Degraded cloud

If `GET /agent/v1/health` fails but provider BYOK works:

- Prompt user: "Cloud unavailable. Continue with your API key?"
- Disable: sync, audit upload, policy refresh (use cached policy max 24 h)
- Enable: direct provider runs

## Degraded model

| Condition | Action |
|-----------|--------|
| HTTP 429 | Exponential backoff; show retry countdown |
| HTTP 5xx | Retry 2×; then fail run with partial text preserved |
| Timeout (180 s) | Cancel; offer continue with shorter context |
| Context overflow | Auto-compact (Cersei); if still overflow, truncate tool results |

## Cached policy

Desktop caches last `GET /agent/v1/policies/effective` with ETag.

| Age | Behavior |
|-----|----------|
| < 24 h | Enforce cached policy |
| ≥ 24 h offline | Stricter: read-only tools only until refresh |
| ≥ 7 d offline | Block write tools until online |

## Sync conflict (Phase 23)

Last-write-wins for message content with conflict marker in UI; plans use file-level merge.

## Local-only mode (enterprise air-gap)

Documented in [../06-enterprise/07-air-gapped-and-self-hosted.md](../06-enterprise/07-air-gapped-and-self-hosted.md):

- No cloud calls after initial license validation (optional).
- BYOK or on-prem inference endpoint only.
- Audit export to local file.

## User messaging

All degraded states use error taxonomy codes — see [../04-specs/15-error-taxonomy.md](../04-specs/15-error-taxonomy.md):

- `EURY_OFFLINE`
- `EURY_GATEWAY_UNAVAILABLE`
- `EURY_POLICY_STALE`
- `EURY_MODEL_RATE_LIMITED`

## Related documents

- [03-cloud-architecture.md](03-cloud-architecture.md)
- [../07-ops/06-runbooks.md](../07-ops/06-runbooks.md)
