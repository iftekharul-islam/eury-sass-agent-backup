# Backup and Disaster Recovery

Spec-Version: 1.1.0

## Objectives

| Scope | RPO | RTO |
|---|---|---|
| Cloud control plane (API) | n/a (stateless) | 1 hour |
| PostgreSQL (`Agent*` tables) | 5 min (PITR) | 4 hours |
| Redis | Not backed up (derived state) | minutes (rebuild) |
| Object storage (releases) | 0 (versioned) | 1 hour |
| Object storage (audit archive) | 0 (versioned, object-locked) | 24 hours |
| Desktop local data | User-controlled | n/a |

The architecture limits blast radius: the agent loop is local, so a total cloud outage degrades features rather than stopping work ([offline and degraded modes](../02-architecture/06-offline-and-degraded-modes.md)).

## PostgreSQL

| Control | Detail |
|---|---|
| Full backups | Nightly, encrypted at rest, retained 30 days |
| PITR | WAL archiving with 5-minute recovery granularity, 7-day window |
| Weekly retention | One weekly full retained 90 days; one monthly retained 12 months |
| Location | Primary region plus cross-region copy for enterprise tiers |
| Encryption | AES-256 at rest; backup credentials separate from app credentials |
| Verification | Automated nightly restore of the latest backup into a scratch instance, followed by a schema and row-count check on `Agent*` tables |
| Drill | Quarterly full restore drill with timing recorded against the RTO |

Because every Agent migration is additive, an application rollback does not require a schema rollback — which removes the most common cause of a botched recovery.

## Redis

Intentionally not backed up. It holds quota counters, rate limits, abort handles, and caches. On loss:

1. Counters rebuild from the `AgentUsageCounter` rollup (up to one flush interval of usage may be re-counted, biased toward the user's favor).
2. Rate limits start empty; the API applies a conservative cold-start limit for 5 minutes.
3. In-flight stream abort handles are lost, so orphaned upstream streams are reaped by the 300-second timeout.

## Object storage

| Bucket/prefix | Controls |
|---|---|
| `agent-releases/` | Versioning on, lifecycle keeps all released versions, cross-region replication, public read via CDN only |
| Audit archive | Versioning on, object lock (compliance mode) for the contracted retention, no delete role granted to the app |
| Crash reports | 90-day lifecycle expiry |

Deleting a release object never deletes the `AgentRelease` row, and the admin console refuses to activate a release whose artifacts are missing — so a storage mistake cannot silently produce a broken update manifest.

## Secrets and keys

| Item | Backup approach |
|---|---|
| `AGENT_*` service secrets | Held in the platform secret manager with its own versioning and recovery window |
| Update manifest root key | Offline HSM, sealed backup in a separate physical location, split custody |
| Platform signing certificates | HSM/token, recovery documented in the key ceremony record |
| Policy signing key | HSM, rotation planned one release ahead |

Losing the update root key without a backup would permanently break auto-update for installed clients, so its custody procedure is reviewed annually.

## Desktop local data

| Data | Responsibility |
|---|---|
| Workspace source code | User's git remote — the agent is never the only copy |
| `db/agent.sqlite` | App keeps a rolling snapshot before each schema migration (14 days) |
| Manual export | Settings → Data → Export produces a zip of conversations, runs, plans, and settings as JSON |
| Optional cloud sync | When enabled, conversations sync to the platform and inherit its backups |
| Checkpoints | Local only, pruned by age/size; not a backup product |
| Keychain items | Not exported by design; re-login is the recovery path |

The app never presents itself as a backup system for user code. Checkpoints protect against a bad agent edit, not against disk loss.

## Disaster scenarios

| Scenario | Impact | Recovery |
|---|---|---|
| API instances lost | No login, no managed inference; BYOK keeps working within grace | Redeploy stateless instances (RTO 1 h) |
| Region loss | Full cloud outage in that region | Fail over to secondary: promote DB replica, repoint DNS, replicate release bucket (enterprise tier) |
| Postgres corruption | Auth, policy, audit ingest down | PITR to just before corruption; replay client audit queues (clients retain 30 days) |
| Redis loss | Quota/rate limits reset | Rebuild from rollup; conservative limits for 5 min |
| Release bucket loss | Downloads and updates fail | Restore from versioning/replication; manifests remain intact |
| Audit archive loss | Compliance exposure | Restore from object versioning; if unrecoverable, notify affected orgs — this is a reportable event |
| Signing key compromise | Update channel untrusted | RB-11 in [runbooks](06-runbooks.md) |
| Accidental data deletion by a migration | Additive-only policy makes this unlikely | PITR restore; the migration review checklist is the primary control |
| Provider outage | Managed inference degraded | Model failover within policy; status communication |

## Recovery procedure (Postgres PITR)

1. Declare the incident and freeze writes (enable maintenance mode on the API).
2. Identify the target timestamp from logs and the incident timeline.
3. Restore into a **new** instance; never restore over the live one.
4. Run `prisma migrate status` and verify `Agent*` schema and row counts.
5. Run application smoke tests against the restored instance (login, policy fetch, audit ingest).
6. Repoint the application, lift maintenance mode, and monitor for 60 minutes.
7. Ask clients to flush audit queues; verify hash-chain continuity per device.
8. Write the post-incident report with the actual RPO and RTO achieved.

## Testing schedule

| Test | Frequency |
|---|---|
| Automated restore verification | Nightly |
| PITR to an arbitrary timestamp | Monthly |
| Full region failover (staging) | Quarterly |
| Signing key recovery walkthrough (tabletop) | Annually |
| Desktop export/import round trip | Every release |
| Audit queue replay after a simulated outage | Quarterly |

A control that has not been tested in its interval is treated as broken and appears on the reliability review.

## Related documents

- [Runbooks](06-runbooks.md)
- [Incident response](../03-security/09-incident-response.md)
- [Cloud data model](../04-specs/07-cloud-data-model.md)
- [Audit and retention](../06-enterprise/04-audit-and-retention.md)
