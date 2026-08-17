# Telemetry Specification

Spec-Version: 2.0.0

Product analytics from the desktop app. **Default: off.** Nothing in this document happens until the user turns it on, and everything in it is designed so that turning it on is a safe decision for someone working on proprietary code.

## Hard rules

| # | Rule |
|---|---|
| N1 | Telemetry is opt-in. A fresh install sends zero analytics events |
| N2 | The consent prompt appears once, is skippable, and defaults to off with no dark patterns ([approval UX](../05-ui/05-approval-and-trust-ux.md)) |
| N3 | Only events listed in this document may be sent. Adding one requires a documented change and a review |
| N4 | Only the property types listed may be sent: enums, booleans, integers, durations, and hashes. Never free-form strings from user content |
| N5 | No prompt text, completion text, code, file path, file name, repository name, branch name, URL, command, or environment variable value — ever |
| N6 | Payloads are inspectable: the user can view the exact JSON of recent events in Settings |
| N7 | Telemetry is best-effort. It never blocks, retries aggressively, or degrades the app when unreachable |
| N8 | Analytics and audit are separate systems with separate consent, separate transport, and separate retention ([audit](../06-enterprise/04-audit-and-retention.md)) |

N8 is the distinction users and auditors care about most: audit is a compliance obligation an organization opts into, analytics is a product-improvement favor an individual opts into. Conflating them would make both untrustworthy.

## Identity

| Id | Value | Lifetime | Reset |
|---|---|---|---|
| `installId` | Random UUIDv4 | Per installation | On "reset telemetry id" or reinstall |
| `sessionId` | Random UUIDv4 | Per app launch | Every launch |
| `userHash` | `sha256(userId + installSalt)`, first 16 hex chars, only when signed in | Stable per install | With `installId` |
| `orgHash` | `sha256(orgId + installSalt)`, first 16 hex chars | Stable | With `installId` |

`installId` is not derived from any hardware identifier, MAC address, or machine name, so it cannot be correlated across reinstalls or joined to a device fingerprint. The salt is local and never transmitted, which means the hashes cannot be reversed server-side into user or org ids.

## Event catalog

Every event carries the common envelope and nothing else beyond its listed properties.

```typescript
interface TelemetryEnvelope {
  event: string;
  ts: string;                       // RFC3339, second precision (not milliseconds)
  installId: string; sessionId: string;
  userHash?: string; orgHash?: string;
  app: { version: string; channel: "stable"|"beta"|"canary"; buildSha: string };
  os: { name: "macos"|"windows"|"linux"; version: string; arch: "x64"|"arm64" };
  locale: string;                   // language tag only, e.g. "en", not "en-US-x-…"
  props: Record<string, string | number | boolean>;
}
```

### Lifecycle

| Event | Properties |
|---|---|
| `app.started` | `coldStartMs`, `firstLaunch`, `updatedFrom?`, `sandboxKind`, `airGapped` |
| `app.quit` | `sessionDurationMs`, `runCount`, `crashOnPreviousExit` |
| `app.updated` | `fromVersion`, `toVersion`, `durationMs`, `channel` |
| `app.update_failed` | `stage`, `errorCode`, `attempt` |

### Usage

| Event | Properties |
|---|---|
| `workspace.opened` | `fileCountBucket`, `languageCount`, `isGitRepo`, `indexState`, `openMs` |
| `run.started` | `mode`, `route`, `modelFamily`, `hasAttachments`, `promptTokenBucket` |
| `run.completed` | `mode`, `status`, `stopReason`, `turns`, `toolCallCount`, `durationMsBucket`, `ttfbMs`, `promptTokenBucket`, `completionTokenBucket`, `filesChangedBucket`, `compactionCount`, `retrievalUsed` |
| `tool.executed` | `toolName`, `toolClass`, `ok`, `durationMsBucket`, `errorCode?`, `truncated` |
| `approval.decided` | `toolClass`, `risk`, `decision`, `scope?`, `waitMsBucket`, `source` |
| `mode.switched` | `from`, `to` |
| `plan.built` | `stepCount`, `completedSteps`, `outcome`, `durationMsBucket` |
| `subagent.used` | `role`, `ok`, `turns`, `costBucket` |
| `checkpoint.restored` | `scope`, `fileCountBucket`, `hadConflicts` |
| `mcp.tool_called` | `serverHash`, `ok`, `durationMsBucket` |
| `memory.proposal_decided` | `kind`, `accepted`, `confidenceBucket` |
| `command.invoked` | `commandId`, `source` (`palette`\|`shortcut`\|`menu`\|`button`) |
| `feature.discovered` | `featureId`, `sessionRunCount` |

`toolName` is an enum from a closed set. `serverHash` is a hash of the MCP server name, because server names can be project-specific and therefore identifying. `commandId` is a closed set from the command registry ([keyboard](../05-ui/08-keyboard-and-command-palette.md)).

### Quality

| Event | Properties |
|---|---|
| `error.surfaced` | `errorCode`, `domain`, `recoverable`, `surface`, `retried` |
| `perf.slow_operation` | `operation`, `durationMs`, `p` (which threshold was exceeded) |
| `ui.frame_drop` | `surface`, `droppedFrames`, `durationMs` |
| `crash.reported` | `crashId`, `stackHash`, `module`, `wasDuringRun` |

### Bucketing

Continuous values are bucketed before leaving the device so an individual value cannot be a fingerprint:

| Family | Buckets |
|---|---|
| Duration | `<100ms`, `<500ms`, `<2s`, `<10s`, `<60s`, `<5m`, `<30m`, `>=30m` |
| Tokens | `<1k`, `<5k`, `<20k`, `<50k`, `<100k`, `<200k`, `>=200k` |
| File count | `1`, `2-5`, `6-20`, `21-100`, `101-1k`, `1k-10k`, `>10k` |
| Cost | `<$0.01`, `<$0.10`, `<$1`, `<$10`, `>=$10` |

Two exceptions are sent raw because they are the numbers we actually optimize and buckets would destroy the signal: `coldStartMs` and `ttfbMs`. Both are clamped to a 120-second maximum.

## Transport

| Aspect | Behavior |
|---|---|
| Batching | Queued locally, flushed every 5 min or at 50 events, plus once on clean quit |
| Endpoint | `POST {EURY_AGENT_CLOUD_URL}/agent/v1/telemetry` — our own endpoint, so no third-party SDK sees the data |
| Compression | gzip |
| Size cap | 256 KB per batch; overflow drops the oldest events |
| Retry | 3 attempts with backoff, then the batch is discarded. Telemetry never accumulates unboundedly |
| Local queue cap | 500 events or 24 hours, whichever comes first |
| Offline | Queued within the caps, then dropped. Never a blocking concern |
| Sampling | 100% for lifecycle and error events; `ui.frame_drop` at 10%; `feature.discovered` once per feature per install |
| Timing | Never during an active run, so telemetry cannot affect measured latency |

## Crash reporting

Separate consent from analytics, because a crash report is more sensitive than an event count.

| Control | Detail |
|---|---|
| Consent | Opt-in, independent toggle. A crash with consent off produces a local file and an offer to attach it to a bug report |
| Content | Stack trace with symbols, module, app and OS version, the last 20 breadcrumb event **names**, and a `stackHash` |
| Stripped | Absolute paths reduced to the file name; user names removed from paths; heap and register dumps excluded; environment excluded; the last prompt excluded |
| Minidumps | Not collected. They can contain arbitrary process memory, which may include code and keys |
| Deduplication | By `stackHash`, so one crash loop is one report, not hundreds |
| Rate limit | 5 reports per session |
| Review | The user can view the exact report contents before it is sent, on first crash |

## Cloud-side metrics (not telemetry)

The managed gateway records operational data as a matter of running the service, independent of desktop telemetry: user and org ids, model, token counts, latency, status, and request id. That is operational necessity documented in [observability](../07-ops/05-observability-and-slos.md) and [privacy](../03-security/07-privacy-and-data-residency.md), it is disclosed in the privacy documentation, and it never includes prompt or completion content. BYOK traffic never passes through our infrastructure and therefore produces none of it.

## Enterprise controls

| Policy field | Effect |
|---|---|
| `telemetry.mode: "off"` | Analytics disabled and the toggle is locked, with the reason shown |
| `telemetry.mode: "user_choice"` | Default; the user decides |
| `telemetry.mode: "required"` | Enabled and locked; requires the organization to have disclosed it to its users |
| `telemetry.crashReports: false` | Crash reporting disabled and locked |
| `telemetry.endpoint` | Self-hosted collector URL |

An organization can require audit while forbidding analytics, which is the common enterprise posture and works because of N8. Air-gapped installations send nothing, unconditionally, regardless of policy.

## User controls

Settings → Privacy shows: independent analytics and crash toggles, a plain-language list of what each sends, a live viewer for the last 100 queued or sent events as raw JSON (N6), a "reset telemetry id" action, "clear queued events", and a link to the privacy documentation. Turning telemetry off drops the local queue immediately rather than sending a final batch.

## Retention

| Data | Retention |
|---|---|
| Raw analytics events | 90 days |
| Aggregated metrics | 25 months |
| Crash reports | 180 days |
| Local queue | 24 hours maximum |

## Conformance tests

| ID | Test |
|---|---|
| T1 | A fresh install makes zero analytics network calls, verified by an egress test |
| T2 | Every emitted event validates against a schema allowlist; an undocumented event fails the build |
| T3 | Property-value fuzzing: no free-form string field can carry user content (N4) |
| T4 | A seeded corpus of paths, prompts, secrets, and repo names never appears in any payload (N5) |
| T5 | Bucketing is applied to every continuous field except the two documented exceptions |
| T6 | Queue caps hold under 10 000 rapid events; memory stays flat |
| T7 | A dead collector endpoint causes no user-visible delay or error |
| T8 | Crash reports contain no absolute paths, environment values, or memory dumps |
| T9 | `telemetry.mode: "off"` locks the UI toggle and blocks all sends |
| T10 | Air-gapped mode sends nothing even with telemetry explicitly enabled |
| T11 | The event viewer shows exactly what was transmitted, byte for byte |
| T12 | Resetting the telemetry id changes `installId`, `userHash`, and `orgHash` together |

## Related documents

- [Privacy and data residency](../03-security/07-privacy-and-data-residency.md)
- [Observability and SLOs](../07-ops/05-observability-and-slos.md)
- [Audit and retention](../06-enterprise/04-audit-and-retention.md)
- [Cloud API contract](06-cloud-api-contract.md)
