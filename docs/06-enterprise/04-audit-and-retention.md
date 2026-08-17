# Audit and Retention

Spec-Version: 1.1.0

Audit answers "what did the agent do on this machine, under whose account, under which policy" — without becoming a copy of the user's source code.

## Event catalog

| Category | Event types |
|---|---|
| Run | `run_start`, `run_complete`, `run_aborted`, `run_failed` |
| Tool | `tool_start`, `tool_end`, `tool_denied`, `tool_timeout` |
| Permission | `permission_requested`, `permission_granted`, `permission_denied`, `permission_revoked`, `approval_timeout` |
| Policy | `policy_applied`, `policy_denied`, `policy_stale_blocked`, `policy.exception_requested`, `policy.exception_granted`, `policy.workspace_override_rejected` |
| Model | `model_call` (metadata only), `model_denied`, `gateway_throttled` |
| Auth | `auth.device_start`, `auth.device_approved`, `auth.device_denied`, `auth.refresh_rotated`, `auth.refresh_reuse`, `auth.logout` |
| Device | `device.enrolled`, `device.revoked`, `device.key_rotated` |
| Identity | `sso.assertion_accepted`, `sso.assertion_rejected`, `scim.*`, `identity.deprovisioned` |
| Data | `checkpoint_created`, `checkpoint_restored`, `export_created`, `sync_enabled`, `sync_disabled` |
| MCP | `mcp.server_connected`, `mcp.tool_invoked`, `mcp.server_rejected` |
| Admin | `admin.policy_activated`, `admin.release_published`, `admin.release_rolled_back`, `admin.audit_exported`, `admin.device_revoked` |
| Integrity | `audit.gap_detected`, `audit.signature_invalid` |

## Event envelope

```json
{
  "id": "0192f0c1-…",
  "seq": 10482,
  "prevHash": "sha256:…",
  "eventType": "tool_end",
  "severity": "info",
  "occurredAt": "2026-08-16T03:41:22.184Z",
  "userId": "cuid",
  "organizationId": "cuid",
  "deviceId": "cuid",
  "runId": "01J…",
  "policyVersion": 7,
  "appVersion": "1.0.0",
  "workspaceHash": "sha256:…",
  "payload": {
    "tool": "write_file",
    "toolClass": "write",
    "argsHash": "sha256:…",
    "path": "src/auth/session.ts",
    "pathHash": "sha256:…",
    "bytesWritten": 1284,
    "ok": true,
    "durationMs": 12,
    "exitCode": null
  }
}
```

| Field | Purpose |
|---|---|
| `seq` + `prevHash` | Per-device hash chain: makes deletion or reordering detectable |
| `argsHash` | Lets an auditor prove two operations were identical without storing arguments |
| `workspaceHash` | Correlates activity per repo without revealing the path |
| `policyVersion` | Proves which rules were in force |

## What is never recorded

- Prompt or completion text (unless the org enables full-payload audit, which is off by default and requires a signed acknowledgement).
- File contents, diffs, or patch bodies.
- Command output.
- Provider API keys, tokens, or anything read from the keychain.
- Absolute paths outside the workspace (hash only).

`path` is included only when `data.auditIncludePaths` is true; otherwise only `pathHash`. Redaction runs **before** the event is written to the local queue, so unredacted data never touches disk ([privacy](../03-security/07-privacy-and-data-residency.md)).

## Local queue and upload

| Concern | Rule |
|---|---|
| Storage | `audit_queue` table in the local encrypted SQLite |
| Durability | Written synchronously before the tool result is returned to the model |
| Batching | Up to 500 events or 2 MB, flushed every 30 s or on run completion |
| Signing | Ed25519 over canonical JSON with the device key registered at enrollment |
| Transport | `POST /agent/v1/audit/batch`, idempotent on `events[].id` |
| Retry | Exponential backoff 1s → 5 min, cap 24 attempts, then surfaced in Settings → Audit |
| Backlog cap | 100 MB or 30 days; when `auditUploadRequired`, exceeding the cap blocks write/execute tools instead of dropping events |
| Ordering | Server tolerates out-of-order arrival, detects gaps via `seq` and emits `audit.gap_detected` |
| Offline | Queue grows; nothing is lost, nothing is silently discarded |

## Retention

| Tier | Local | Cloud | Archive |
|---|---|---|---|
| Free | 7 days | none | none |
| Pro | 30 days | none | none |
| Business | 30 days | 90 days | none |
| Enterprise | 30 days | 1–24 months configurable | object storage, up to 7 years |

| Rule | Detail |
|---|---|
| Purge job | Nightly, deletes cloud rows past retention, writes a purge summary event |
| Legal hold | Suspends purge for named users/orgs; setting and clearing it is audited |
| Immutability | No update or delete code paths; purge is the only deletion and is batch-logged |
| Archive | Monthly Parquet/NDJSON export to object storage with versioning and object-lock |
| Deletion requests | GDPR erasure removes content-bearing fields and pseudonymizes `userId`, preserving the hash chain's integrity |

## Access and export

| Route | Permission |
|---|---|
| `GET /agent/v1/admin/audit` | `agent:view_audit` |
| `GET /agent/v1/admin/audit?format=csv` | `agent:view_audit` |

Filters: `userId`, `deviceId`, `runId`, `eventType[]`, `severity`, `occurredAt` range, `workspaceHash`. Pagination is cursor-based; exports above 50k rows are queued and delivered as a signed, expiring download link. Every export is itself audited (`admin.audit_exported`) with the filter set.

## SIEM integration

| Mode | Detail |
|---|---|
| Webhook | POST batches to a customer endpoint, HMAC-signed, at-least-once, retried 24 h |
| Splunk / Elastic | NDJSON over HTTP Event Collector |
| S3 pull | Customer-owned bucket, hourly NDJSON objects |
| Format | Native envelope, or OCSF-mapped when `format=ocsf` |

Ships in Phase 25.

## Integrity verification

An admin can run "Verify chain" per device: the server recomputes `prevHash` across `seq` and reports the first break. Verification output is downloadable as evidence for an audit. A broken or missing chain is a security event, not a warning — it raises `audit.gap_detected` at `critical` severity and appears on the admin dashboard.

## Compliance mapping

| Control | Satisfied by |
|---|---|
| SOC 2 CC7.2 (monitoring) | Full tool/permission event coverage + SIEM export |
| SOC 2 CC6.1 (access control) | Auth/device/permission events with policy version |
| ISO 27001 A.12.4 (logging) | Hash-chained, immutable, retained logs |
| GDPR Art. 30 (records) | Metadata-only default with documented fields |
| GDPR Art. 17 (erasure) | Pseudonymizing erasure preserving integrity |

Baseline details: [compliance baseline](../03-security/08-compliance-baseline.md).

## Testing

| Test | Assertion |
|---|---|
| Redaction | No event body ever contains file contents, secrets, or command output (fuzzed corpus) |
| Chain | Tampering with any queued event is detected by verification |
| Idempotency | Replaying a batch yields `duplicates`, never double counts |
| Signature | Wrong key, wrong body, and replayed signature all rejected |
| Backlog | `auditUploadRequired` + full backlog blocks tools instead of dropping events |
| Retention | Purge respects tier and legal hold; purge itself is logged |

## Delivery

Event emission, the hash-chained local queue, and redaction ship in Phase 7 alongside the policy engine. Cloud ingest, chain verification, retention, export, and SIEM delivery ship in Phase 25.

## Related documents

- [Cloud API contract](../04-specs/06-cloud-api-contract.md)
- [Telemetry spec](../04-specs/14-telemetry-spec.md)
- [Privacy and data residency](../03-security/07-privacy-and-data-residency.md)
- [Workspace policies](03-workspace-policies.md)
