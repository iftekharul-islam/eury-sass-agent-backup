# Definition of Done

Spec-Version: 1.2.0

Three gates: feature, phase, release. Each is a hard checklist — "mostly done" is not done.

## Feature done

- [ ] Behavior matches the spec; if the spec was wrong, the spec was updated in the same PR
- [ ] New or changed contracts (events, IPC, HTTP, DB) have updated golden fixtures
- [ ] Unit tests for new logic; a regression test for every fixed bug
- [ ] Coverage did not drop for the touched crate or module
- [ ] Errors use codes from the [error taxonomy](../04-specs/15-error-taxonomy.md); no bare strings
- [ ] No new `unwrap`, `expect`, or `panic!` in tool, IPC, or run-loop paths
- [ ] Filesystem and process access goes through `agent-sandbox`
- [ ] New tool or capability is deny-by-default and appears in the policy schema
- [ ] Security-impact section links affected `A-*`/`ACT-*`/`B-*`/`T-*`/`C-*`/`TEST-*` identifiers
- [ ] Canonical security schemas validate every changed policy, decision, grant, trust, workspace-state, sandbox-capability, and corpus example
- [ ] No privileged child egress occurs without a separate Network decision bound to the approval shape
- [ ] Dependency/action/tool changes document pins, transitive/lifecycle-script/license/advisory review and rollback
- [ ] Any security exception names owner, approver, compensating control, expiry, and removal target; expired exceptions fail CI
- [ ] Keyboard path exists; `jest-axe` clean; focus order verified
- [ ] Strings are i18n keys, not literals; no key orphans
- [ ] Naming follows the [naming and migration map](../00-overview/05-naming-and-migration-map.md) — no legacy `code` identifiers
- [ ] Backend changes stay inside `modules/agent/` and pass the isolation check
- [ ] Telemetry and audit events added where the behavior is security- or cost-relevant, with redaction
- [ ] Performance impact measured if on a hot path
- [ ] Changelog entry written for user-visible changes
- [ ] Documentation quality gate passes; affected [traceability](../00-overview/06-document-lifecycle-and-traceability.md) row and lifecycle state are updated
- [ ] Screenshot or short clip in the PR for visual changes
- [ ] Two reviewers for security-sensitive areas (sandbox, policy, auth, IPC surface, update path)

## Phase done

- [ ] Every deliverable in the phase file is complete and demoable
- [ ] Every exit criterion is objectively verified, not asserted
- [ ] The phase's test plan has been executed and its suites are in CI
- [ ] The phase's metrics are collected and recorded against targets
- [ ] Security checklist for the phase passed (Phase 2 onward)
- [ ] Docs updated: any spec that drifted during implementation is corrected
- [ ] ADR written for any decision that deviated from the plan
- [ ] No new item on the deferred list without an owner and a target phase
- [ ] Demo recorded and reviewed by product
- [ ] Risks in the [risk register](../09-roadmap/risk-register.md) re-scored
- [ ] Open questions raised by the phase added to [open questions](../09-roadmap/open-questions.md)

## Release done (per version)

- [ ] Full release checklist in [release management](../07-ops/08-release-management.md) complete
- [ ] Per-release [security checklist](04-security-testing.md) complete
- [ ] Eval gates met: 0 unsafe results, pass rate ≥ 85%, injection and tool-discipline categories at 100%
- [ ] Benchmarks within regression policy on all three platforms
- [ ] E2E suites green on macOS, Windows, Linux
- [ ] Signed, notarized artifacts verified; manifest signature validated by a client build
- [ ] Fresh install and N−1 upgrade smoke passed
- [ ] Compatibility evidence passes for every changed API, event, policy, model catalog, and local-schema surface
- [ ] Rollback tested and the previous release id recorded
- [ ] Release notes published; enterprise admins notified of policy-relevant changes

## GA done

- [ ] Phases 0–29 exit criteria met, or explicitly deferred with an ADR naming the tradeoff
- [ ] Eval suite ≥ 80 tasks; gates met on every supported model
- [ ] External pentest complete; all critical and high findings closed with a retest report
- [ ] SOC 2 control evidence collected for the [compliance baseline](../03-security/08-compliance-baseline.md)
- [ ] Accessibility: WCAG 2.2 AA verified, including a manual screen-reader pass on all three platforms
- [ ] SLOs instrumented with dashboards, alerts, and a runbook link per alert
- [ ] Backup and DR drills passed within their intervals
- [ ] Air-gapped egress report produced for the offline build profile
- [ ] Enterprise surfaces complete: SSO, SCIM, policy distribution, audit export, budgets, admin console
- [ ] Docs published; every spec's `Spec-Version` matches the shipped behavior
- [ ] Migration guide from `code-old` written and tested by someone who was not involved in writing it
- [ ] Legacy sunset plan communicated with dates
- [ ] Support runbooks and on-call rotation staffed; one game-day exercise completed
- [ ] Provider outage, audit backlog, and encrypted-local-store recovery exercises passed within the required interval
- [ ] Signed installers available for all platforms with checksums, SBOM, and provenance

## Explicitly not "done"

A change is not done because it works on the author's machine, because tests were skipped with a note, because the docs will be updated later, or because a security check was deferred to a follow-up ticket. These are the four ways this checklist gets quietly bypassed, so reviewers are expected to name them.

## Related documents

- [Test strategy](01-test-strategy.md)
- [Agent eval harness](02-agent-eval-harness.md)
- [Security testing](04-security-testing.md)
- [Release management](../07-ops/08-release-management.md)
