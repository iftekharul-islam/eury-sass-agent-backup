# Auto-Update and Rollback

Spec-Version: 1.1.0

## Check flow

```
1. On startup (after 30 s idle) and every 24 h: GET /agent/v1/releases/latest?platform=…&arch=…&channel=…
2. Verify manifest signature against the embedded channel public key
3. Compare semver against the running version
4. installed < minSupported        → blocking update required
   installed < latest && mandatory → blocking update required
   installed < latest              → non-blocking prompt (or silent download if enabled)
   installed >= latest             → no-op
```

| Rule | Value |
|---|---|
| Check timeout | 5 s, failures are silent |
| Backoff | Failed checks back off to 6 h, then 24 h |
| Never | Check while a run is active, on metered connections, or when `EURY_AGENT_OFFLINE` is set |
| Staged rollout | Server decides eligibility from a stable hash of `deviceId`; the client does not self-select |
| Policy override | `update.autoUpdate = false` and `update.channel` in org policy win over user settings |

## Download and apply

| Step | Detail |
|---|---|
| Download | Background, resumable, to `cache/updates/`, progress in the status bar |
| Verify | SHA-256 against the manifest, then platform signature verification (`codesign`, `signtool`, GPG) |
| Reject | Any mismatch deletes the file, reports `EURY_UPDATE_INTEGRITY_FAILED`, and disables auto-update until the next manifest change |
| Apply | Only at a safe point: no active run, no unsaved editor buffer, no pending approval |
| Prompt | "Restart to update" with "Later"; the update is never applied under an active run |
| macOS | Tauri updater replaces the `.app` bundle in place from the signed `.app.tar.gz`; `.dmg` is for fresh installs |
| Windows | `msiexec /i … /qb` per-user, app relaunches itself |
| Linux (deb) | Notifies and opens the package manager; no silent privileged install |
| Linux (AppImage) | Replaces the AppImage after verifying the signature |
| State migration | SQLite migrations run on first launch of the new version, inside a transaction, after a pre-migration backup |

## Blocking updates

When `installed < minSupported`, the app shows a modal that cannot be dismissed, allows viewing/exporting local data, and blocks agent runs. This exists so a security fix can be enforced. Raising `minSupported` is an audited admin action with typed confirmation ([admin console](../06-enterprise/06-admin-console-spec.md)).

## Post-update health check

On first launch after an update, the app runs a self-check: SQLite opens and migrates, keychain items are readable, the sandbox initializes, the index opens, and one IPC round-trip succeeds. Failure marks the update unhealthy, reports it (opt-in), and offers reverting to the previous version.

Two failed launches within 10 minutes trigger automatic safe mode (`--safe-mode`) with a revert prompt.

## Rollback

### User-side

The previous version's installer and a snapshot of `db/agent.sqlite` are retained for 14 days. Settings → About → "Revert to previous version" verifies the cached installer's signature, restores the pre-migration database snapshot, and reinstalls. Downgrading across a schema migration without the snapshot is refused rather than risking corruption.

### Fleet-side

1. Admin activates the previous `AgentRelease` for the channel (one click).
2. Manifest flips within 60 s (`Cache-Control: max-age=60`).
3. Clients already updated stay on the newer version unless `minSupported` is lowered — the fleet action stops the spread, it does not force a downgrade.
4. If the new version is actively harmful, publish a hotfix and raise `minSupported` to it.

Runbook: [runbooks](06-runbooks.md).

## Channels

| Channel | Audience | Cadence |
|---|---|---|
| `stable` | Default | Bi-weekly |
| `beta` | Opt-in, internal dogfood | Weekly |
| `canary` | Team only | Per merge to `main` |

Channel is chosen in Settings → Updates, overridable by org policy. Moving from a higher to a lower channel does not downgrade; it waits until `stable` catches up.

## Telemetry

Update events (`update_check`, `update_downloaded`, `update_applied`, `update_failed`, `update_reverted`) carry version, channel, platform, and error code — no user content. They feed the release health dashboard: adoption curve, failure rate by platform, revert rate, and crash rate per version ([observability](05-observability-and-slos.md)).

Release-health gates for promoting a staged rollout: update failure rate < 1%, crash-free sessions > 99.5%, no new SEV1/2.

## Air-gapped

Auto-update is disabled entirely under `EURY_AGENT_OFFLINE`. Administrators sideload signed installers and enforce a minimum version through the local signed policy file ([air-gapped and self-hosted](../06-enterprise/07-air-gapped-and-self-hosted.md)).

## Testing

| Test | Assertion |
|---|---|
| Happy path | N−1 → N on all three platforms, settings and conversations preserved |
| Integrity | Tampered artifact and tampered manifest both rejected |
| Wrong signature | Manifest signed by an unknown key is rejected |
| Interrupted download | Resumes; partial file never applied |
| Interrupted apply | App still launches (old or new, never broken) |
| Migration failure | Rolls back the transaction, restores the snapshot, app remains usable |
| Blocking update | Run start is blocked; data export still works |
| Revert | Restores the previous version and its database snapshot |
| Rollout | Stable hash bucketing is deterministic per device |

## Related documents

- [Packaging, signing, notarization](03-packaging-signing-notarization.md)
- [Release management](08-release-management.md)
- [Local data model](../04-specs/05-local-data-model.md)
- [Cloud API contract](../04-specs/06-cloud-api-contract.md)
