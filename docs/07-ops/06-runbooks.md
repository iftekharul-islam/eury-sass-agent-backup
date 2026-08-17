# Runbooks

Spec-Version: 1.1.0

Each runbook: symptom → triage → mitigation → verification → follow-up. Severity definitions live in [incident response](../03-security/09-incident-response.md).

## RB-01 Gateway 5xx spike

**Symptom:** `agent_http_requests_total{route="/agent/v1/chat/stream",status=~"5.."}` above 2%.

1. Check the upstream inference service health endpoint and its own dashboards.
2. Check `agent_chat_stream_total{outcome="upstream_error"}` split by provider — one provider or all?
3. Check Postgres pool saturation and Redis latency (the guard path touches both).
4. Check recent deploys of the Nest app and of the Agent module specifically.

**Mitigation**

- Single provider degraded: mark it unavailable in the model catalog so clients fail over to another allowed model.
- All providers: scale Nest replicas; if the fault is upstream, publish a status page notice.
- Guard path is the bottleneck: enable the Redis-degraded path (conservative counters) rather than removing quota checks.
- Never disable `AgentAuthGuard` or `AgentUsageGuard` to shed load.

**Verify:** error rate < 0.5% for 15 min; TTFB back within SLO.

**Follow-up:** if a provider caused it, add or tune circuit-breaker thresholds.

## RB-02 Authentication outage

**Symptom:** `/agent/v1/auth/*` errors, users cannot sign in.

1. Confirm `AGENT_JWT_SECRET` and `AGENT_REFRESH_TOKEN_SECRET` are unchanged (a rotation without dual-key verification is the usual cause).
2. Check `AgentAuthSession` and `AgentRefreshToken` table health and Postgres connectivity.
3. Check the device-flow funnel dashboard for the failing step.
4. Check clock skew on the API hosts (JWT `exp`/`nbf` failures).

**Mitigation**

- Bad rotation: restore dual-key verification with the previous secret, then roll forward properly.
- DB issue: follow RB-05.
- Communicate that already-signed-in users keep working for up to 15 min, and BYOK users keep working within their offline grace window.

**Verify:** login success rate back above 99%; refresh rotations succeeding.

**Follow-up:** if reuse detections spiked, treat as a possible security event and escalate.

## RB-03 Refresh-token reuse spike

**Symptom:** `agent_refresh_reuse_total` > 10/hour.

1. Group by user, device, and IP. A single device usually means a client bug (racing refreshes); many devices means possible token theft.
2. Check whether a release changed refresh handling.
3. Inspect the affected `chainId` histories for interleaved rotations.

**Mitigation:** client bug → hotfix and consider temporarily widening the rotation grace to a single-use 10 s window. Suspected theft → revoke the affected chains and devices, notify the org admin, and open a security incident.

**Verify:** reuse rate returns to baseline; affected users can re-login.

## RB-04 Release rollback

**Symptom:** new version causes crashes, update failures, or data issues.

1. Check release health: crash-free sessions, update failure rate, revert rate by version.
2. Decide: stop the spread, or force everyone off the bad version.

**Mitigation**

1. Admin console → Releases → activate the previous release for the channel (stops new downloads within ~60 s).
2. Pause the staged rollout percentage.
3. If the bad version is actively harmful, ship a hotfix and raise `minSupported` to it. Only raise `minSupported` for security or data-loss issues — it hard-blocks users.
4. Post an in-app banner and a status-page note.

**Verify:** manifest serves the previous version; downloads of the bad version stop; crash rate recovers.

**Follow-up:** add the failure mode to the release verification checklist so it cannot ship twice.

## RB-05 Database incident (Agent tables)

**Symptom:** errors touching `Agent*` tables, or a failed migration.

1. `prisma migrate status` against the environment.
2. Identify whether the failure is a migration, a lock, or capacity.

**Mitigation**

- Failed migration: since all Agent migrations are additive, roll the application image back first; the schema is forward-compatible. Then fix the migration and re-apply with `prisma migrate resolve` for a partially applied state.
- Lock contention: identify the blocking query; audit inserts are the highest-volume writer and can be shed to the queue (clients retain events locally).
- Capacity: scale storage, then purge per retention policy.
- Never hand-edit an applied migration.

**Verify:** `prisma migrate status` clean; write paths succeed; audit backlog drains.

**Follow-up:** restore from backup only if data is lost ([backup and DR](07-backup-and-dr.md)).

## RB-06 Audit backlog or gap

**Symptom:** `agent_audit_gap_detected_total` > 0, or ingest failures above 5%.

1. Gap: identify device and `seq` range; determine whether the client was reinstalled (benign, expected chain reset) or events were dropped.
2. Ingest failures: check signature rejections (device key mismatch after re-enrollment) versus 5xx.

**Mitigation:** signature mismatch → verify the device's registered public key and prompt re-enrollment. Server-side failures → scale ingest; clients retain up to 100 MB / 30 days, so recovery is usually lossless. For orgs with `auditUploadRequired`, warn admins that clients will block write/execute tools if the backlog cap is hit.

**Verify:** backlog drains; chain verification passes for affected devices.

**Follow-up:** a genuine unexplained gap is a security incident, not an ops blip.

## RB-07 Quota or budget misfire

**Symptom:** users blocked incorrectly, or spend exceeding budget.

1. Check `agent_quota_denied_total` by reason and compare Redis counters against the `AgentUsageCounter` rollup.
2. Check whether Redis was unavailable (the degraded path is intentionally conservative).
3. Check the price-table version for the affected period.

**Mitigation:** counter drift → recompute from the rollup and reset the Redis key. Wrongly blocked org → raise the budget or grant an audited temporary exception. Overspend → tighten the reservation size and confirm reconciliation.

**Verify:** affected users can run; totals match within one flush interval.

## RB-08 Policy distribution failure

**Symptom:** `/agent/v1/policies/effective` failing, or clients reporting stale policy.

1. Check policy resolution errors and cache hit rate.
2. Validate the active policy document against the schema (a bad activation can break resolution).
3. Check the signing key if enterprise policies are signed.

**Mitigation:** roll back to the previous policy version in the admin console. Clients keep enforcing their cached policy, so the security posture holds; the risk is only stale rules. Warn orgs whose `policyMaxAgeHours` is near expiry, because their clients will fail closed.

**Verify:** 200s with correct ETags; a test device receives the expected version.

## RB-09 Desktop crash loop

**Symptom:** users report the app crashing on launch; crash-free sessions drop.

1. Ask for or fetch the crash report and the version.
2. Reproduce with the same OS and workspace shape if possible.

**Mitigation (user-side, in order)**

1. Relaunch — two failed launches within 10 minutes auto-enter safe mode.
2. `--safe-mode` (read-only tools, no MCP, no index).
3. `--reset-settings` (keeps conversations).
4. Revert to the previous version (restores the pre-migration DB snapshot).
5. Collect diagnostics with `--export-logs` from `EURY_AGENT_DATA_DIR/logs`.

**Fleet-side:** if the crash is version-wide, follow RB-04.

**Verify:** app launches; the self-check passes.

## RB-10 Corrupted local database

**Symptom:** SQLite errors, conversations missing.

1. Check `logs/agent.log` for `SQLITE_CORRUPT` or a failed migration.
2. Confirm whether the WAL is recoverable (`PRAGMA integrity_check`).

**Mitigation:** the app attempts WAL recovery on launch; if that fails it renames the file to `agent.sqlite.corrupt-<ts>`, starts fresh, and offers import from the last snapshot or an export. Queued audit events in a corrupt DB may be unrecoverable — record that in the incident, because for `auditUploadRequired` orgs it is a compliance-relevant gap.

**Verify:** app usable; snapshot restored where available.

## RB-11 Signing or update key compromise

**Symptom:** suspected exposure of a signing key.

1. Treat as SEV1 security. Freeze all releases immediately.
2. Determine which key (platform signing versus update manifest, root versus channel).

**Mitigation:** revoke the certificate with the CA; rotate the channel key and publish a release embedding the new public key; re-sign supported artifacts; consider raising `minSupported` to the re-signed version; notify enterprise customers with the affected checksums. macOS identity changes invalidate keychain items, so the release notes must tell users they will re-authenticate.

**Verify:** old signatures rejected; new release verifies end to end.

## RB-12 Provider outage or key exhaustion

**Symptom:** one provider failing for managed users; BYOK users unaffected.

1. Confirm from the provider's status page and `agent_chat_stream_total{outcome="upstream_error"}`.
2. Check for rate-limit versus auth versus capacity errors.

**Mitigation:** mark affected models unavailable in the catalog so clients see them disabled with a reason; suggest an allowed alternative model; if it is key exhaustion, rotate or add capacity. Do not silently reroute to a different model — model choice is user- and policy-visible.

**Verify:** stream success rate recovers on the remaining providers.

## RB-13 Onboarding a new on-call

Read: this file, [incident response](../03-security/09-incident-response.md), [observability](05-observability-and-slos.md). Confirm dashboard and alert access, run one game-day exercise (a forced gateway error and a release rollback in staging), and confirm the ability to activate a release and roll back a policy in staging.

## Related documents

- [Observability and SLOs](05-observability-and-slos.md)
- [Backup and DR](07-backup-and-dr.md)
- [Incident response](../03-security/09-incident-response.md)
- [Auto-update and rollback](04-auto-update-and-rollback.md)
