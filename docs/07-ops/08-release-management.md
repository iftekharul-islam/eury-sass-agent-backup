# Release Management

Spec-Version: 1.1.0

## Versioning

Semantic versioning for the desktop app; the cloud API is versioned separately as `/agent/v1`.

| Change | Bump |
|---|---|
| Breaking local schema change or removed feature | major |
| New feature, new tool, new UI surface | minor |
| Bug fix, performance, copy | patch |

Tag format `agent-vX.Y.Z` keeps Agent releases distinct from backend and frontend tags. Local SQLite migrations are additive within a major version so a patch downgrade stays safe.

## Trains

| Train | Cadence | Audience | Gate |
|---|---|---|---|
| `canary` | Every merge to `main` | Team | CI green |
| `beta` | Weekly, Thursday | Opt-in + internal dogfood | CI + eval + bench green |
| `stable` | Bi-weekly, Tuesday | Everyone | Full release checklist + 48 h beta soak |
| `hotfix` | On demand | Everyone | SEV1/2 fix, focused test suite |

A missed train waits for the next one. Shipping late on Friday is not a train.

## Release checklist

Owner: release captain (rotating). Recorded in the release issue.

**T−3 days**
- [ ] Cut the release branch from `main`; `main` stays open
- [ ] Version bumped in all five locations ([packaging](03-packaging-signing-notarization.md))
- [ ] Changelog drafted from merged PR titles, grouped by area
- [ ] Migration review: local SQLite and Agent Prisma migrations are additive
- [ ] Cloud compatibility confirmed for N−2 desktop versions
- [ ] Eval harness pass rate ≥ threshold on pinned models
- [ ] Benchmarks within 10% of the previous release
- [ ] Security workflow clean: no high/critical advisory, secret, expired exception, unpinned action/tool, or blocking Semgrep finding
- [ ] Threat/control and policy-schema drift checks pass; residual Critical/High risks have accepted owner/gate
- [ ] Security-rule positive/negative fixtures and current corpus manifests pass
- [ ] Release SBOM/provenance/signature evidence required by the release phase is attached and verified
- [ ] Security-owner review recorded for sandbox, policy, auth, IPC, MCP, data-route, signing, or update changes
- [ ] Accessibility gates green
- [ ] Docs updated for any changed contract

**T−1 day**
- [ ] Beta soaked ≥ 48 h with no SEV1/2
- [ ] Release build produced, signed, notarized, verified on all platforms
- [ ] Fresh-install and N−1 upgrade smoke on macOS, Windows, Linux
- [ ] SBOM, checksums, and provenance attached
- [ ] Rollback plan written (previous release id noted)

**Release day**
- [ ] Activate for 5% of `stable`
- [ ] Watch for 4 h: crash-free > 99.5%, update failure < 1%, no new SEV
- [ ] Expand to 25%, watch 24 h
- [ ] Expand to 100%
- [ ] Publish release notes and update the download page
- [ ] Close the release issue with metrics

**Post-release (T+3 days)**
- [ ] Review adoption, crash, and revert rates
- [ ] File issues for anything the checklist missed
- [ ] Rotate the release captain

## Staged rollout

Eligibility is decided server-side from a stable hash of `deviceId`, so a device does not flip in and out of the cohort between checks. Beta and canary always receive 100% of their channel. Pausing a rollout takes effect within 60 seconds; rollback is RB-04 in [runbooks](06-runbooks.md).

## Feature flags

```json
{
  "multi_agent": false,
  "mcp_marketplace": true,
  "browser_preview": true,
  "cloud_sync": false,
  "byok_fallback_on_gateway_error": true
}
```

| Rule | Detail |
|---|---|
| Source | Defaults compiled in; overridden by `GET /agent/v1/me` response flags; org policy can force a flag off |
| Scope | Per user or per org, never random per session |
| Kill switch | Any risky subsystem (MCP, sub-agents, sync, preview) has a server-side off switch |
| Lifetime | A flag lives at most two minor releases, then it is removed or promoted — tracked in the release issue |
| Security features | Never behind a flag that can enable a weaker path (no flag can disable the sandbox or approvals) |

## Cloud and desktop coordination

| Rule | Detail |
|---|---|
| Compatibility window | Cloud supports the current and previous two desktop minors |
| Order | Cloud ships first (additive), desktop second |
| Additive only | New response fields are optional; the desktop tolerates unknown fields |
| Breaking API change | Requires `/agent/v2` in parallel for ≥ 6 months |
| Forced upgrade | Raising `minSupported` is reserved for security or data-loss issues, and is audited |
| Migration order | Additive DB migration → cloud deploy → desktop release |

## Hotfix process

1. Branch from the released tag, not `main`.
2. Minimal diff, one reviewer plus the release captain.
3. Focused test suite plus the full security workflow.
4. Release as `X.Y.Z+1`, straight to 100% if it fixes a SEV1.
5. Cherry-pick back to `main` the same day.

## Communication

| Audience | Channel |
|---|---|
| Users | In-app release notes on first launch, download page changelog |
| Enterprise admins | Email digest with policy-relevant changes and any `minSupported` change ≥ 7 days ahead (except emergencies) |
| Internal | Release issue with checklist state, metrics, and the rollback decision log |
| Status | Status page for incidents and for forced-update windows |

Release notes state what changed for the user. Internal refactors that are user-invisible are omitted; security fixes are named without exploit detail until users have had time to update.

## Deprecation policy

| Item | Notice |
|---|---|
| Desktop feature removal | 1 minor release with an in-app notice |
| Local schema breaking change | Major version + documented one-way migration + export offered first |
| Cloud API version | 6 months of parallel operation |
| Legacy `/code/*` endpoints | 6 months after Agent GA ([naming and migration map](../00-overview/05-naming-and-migration-map.md)) |
| MCP or tool removal | 1 minor release, policy-visible warning |

## Metrics per release

Adoption curve, crash-free sessions, update success and revert rate, p95 time to first token, eval pass rate, benchmark deltas, new-issue count in the first 72 hours, and support contacts per 1000 active users. Trends are appended to a permanent dataset so regressions are visible across releases, not just within one.

## Related documents

- [CI/CD pipelines](02-ci-cd-pipelines.md)
- [Auto-update and rollback](04-auto-update-and-rollback.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Runbooks](06-runbooks.md)
