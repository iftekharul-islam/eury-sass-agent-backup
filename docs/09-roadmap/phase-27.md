# Phase 27 — Packaging and Release Engineering

Spec-Version: 1.1.0

**Track:** F — Enterprise and GA · **Estimated size:** 2–3 weeks · **Milestone:** —

## Goal

Signed, notarized installers on all platforms, a verified auto-update channel with rollback, release administration, and the enterprise deployment package.

## Why this phase exists here

A desktop app that cannot be trusted to install and update itself has no distribution. This is also where the air-gapped and self-hosted offerings get packaged.

## In scope

- Full `agent-release.yml` pipeline: build, sign, notarize, SBOM, provenance, publish
- Signed update manifests with a root/channel key hierarchy
- Auto-update with integrity verification, safe-point apply, and health check
- Revert-to-previous-version with a pre-migration database snapshot
- Release admin pages: publish, activate, staged rollout, rollback, health
- Channels: stable, beta, canary with server-side rollout bucketing
- Air-gapped build profile with an egress verification report
- Self-hosted container image, `--verify-config` mode, and deployment guide
- Offline licensing with grace and read-only fallback

## Feature IDs

`F-075`

## Out of scope

- App store distribution
- Linux `.rpm` (deferred)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D27.1 | Release pipeline producing verified artifacts for all platforms | [CI/CD](../07-ops/02-ci-cd-pipelines.md) |
| D27.2 | Notarized macOS builds, Authenticode Windows builds, signed Linux artifacts | [packaging](../07-ops/03-packaging-signing-notarization.md) |
| D27.3 | Signed update manifests with an embedded channel public key | [auto-update](../07-ops/04-auto-update-and-rollback.md) |
| D27.4 | Auto-update with verification, health check, and auto safe mode | [auto-update](../07-ops/04-auto-update-and-rollback.md) |
| D27.5 | Revert path with database snapshot restore | [auto-update](../07-ops/04-auto-update-and-rollback.md) |
| D27.6 | Release admin pages with staged rollout and one-click rollback | [admin console](../06-enterprise/06-admin-console-spec.md) |
| D27.7 | SBOM, checksums, and provenance attached to every release | [supply chain](../03-security/06-supply-chain-and-signing.md) |
| D27.8 | Air-gapped profile with a CI egress report | [air-gapped](../06-enterprise/07-air-gapped-and-self-hosted.md) |
| D27.9 | Self-hosted image, config verification mode, and deployment guide | [air-gapped](../06-enterprise/07-air-gapped-and-self-hosted.md) |
| D27.10 | Offline license verification with grace and read-only fallback | [air-gapped](../06-enterprise/07-air-gapped-and-self-hosted.md) |

## Key decisions and design notes

- The update manifest is signed independently of the artifacts, so a CDN compromise alone cannot push an update.
- CI never activates a stable release; activation is a deliberate human step after smoke tests.
- Downgrading across a schema migration without a snapshot is refused rather than risking corruption.
- The air-gapped claim is backed by a CI egress test, not by a promise in a document.

## Contracts touched

- Update manifest format and signature scheme
- Release admin API
- License file format

## Dependencies

- Phase 0 (version consistency)
- Phase 26 (release health telemetry)
- Phase 24 (admin auth)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Signing or notarization failures | Cannot ship | Pipeline exercised on every canary; credentials monitored for expiry with advance alerts |
| Bad update bricking installs | Severe | Health check, auto safe mode, revert path, staged rollout, and a tested rollback |
| Key compromise | Trust loss | HSM custody, split backup, rotation planned a release ahead, and RB-11 |
| macOS identity change invalidating keychain items | Users must re-login | Identity pinned per channel; rotation documented in release notes |

## Test plan

| Layer | Coverage |
|---|---|
| Pipeline | Full release dry-run on a canary tag |
| Integrity | Tampered artifact and tampered manifest both rejected |
| Update | N−1 → N on all platforms; interrupted download and interrupted apply |
| Rollback | Revert restores the previous version and its snapshot |
| Air-gapped | Egress test shows no connection outside the configured endpoint |
| Self-hosted | Container boots with `--verify-config` and passes smoke tests |

## Metrics and targets

| Metric | Target |
|---|---|
| Release pipeline duration | < 60 min end to end |
| Update success rate in test fleet | > 99% |
| Installer size per platform | < 40 MB |
| Unverified artifacts published | 0 |

## Exit criteria

- [ ] Signed, notarized installers produced for all platforms by CI
- [ ] Auto-update verified end to end including integrity rejection cases
- [ ] Revert restores both the app and its data snapshot
- [ ] Staged rollout and one-click rollback work from the admin console
- [ ] SBOM, checksums, and provenance published per release
- [ ] Air-gapped egress report produced; self-hosted image boots and passes smoke tests

## Deferred from this phase

- App store distribution (post-GA)
- Delta updates (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
