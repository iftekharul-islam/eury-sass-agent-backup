# Phase 05 — Workspace and Sandbox

Spec-Version: 1.1.0

**Track:** B — Core runtime · **Estimated size:** 2–3 weeks · **Milestone:** —

## Goal

Implement the containment layer: workspace roots, trust states, path guard, command guard, and OS-level sandboxing — before any tool exists to use it.

## Why this phase exists here

Tools must be born inside a sandbox. Writing the guard after the tools would mean auditing every call site instead of having one enforced chokepoint.

## In scope

- Workspace model: root registration, trust states, git remote detection, availability
- Path guard: canonicalization, symlink resolution, root containment, deny globs
- Open-then-verify semantics to close TOCTOU windows
- Command guard: argv parsing, allow/deny pattern matching, shell-metacharacter handling
- OS sandbox: Seatbelt profile (macOS), Landlock ruleset (Linux), Job object + restricted token (Windows)
- Process supervision: timeouts, output caps, process-group kill, orphan reaping
- Sandbox capability reporting and documented degradation
- Fuzz targets for the path and command parsers

## Feature IDs

`F-008`, `F-025`, `F-067`

## Out of scope

- The tool catalog itself (Phase 6)
- Approval UI (Phase 7)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D5.1 | Workspace registry with trust states and the trust prompt flow | [approval and trust UX](../05-ui/05-approval-and-trust-ux.md) |
| D5.2 | Path guard as the only filesystem entry point in the codebase | [sandbox model](../03-security/02-sandbox-model.md) |
| D5.3 | Command guard with normalized-shape matching | [sandbox model](../03-security/02-sandbox-model.md) |
| D5.4 | Per-platform OS sandbox with capability probing at startup | [sandbox model](../03-security/02-sandbox-model.md) |
| D5.5 | Process supervisor: timeout, output ring buffer, group kill | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D5.6 | Traversal and command corpora wired into a blocking CI suite | [security testing](../08-quality/04-security-testing.md) |
| D5.7 | Fuzz targets for path normalization and command parsing | [security testing](../08-quality/04-security-testing.md) |
| D5.8 | Lint rules banning `std::fs` and `Command::new` outside this crate | [security testing](../08-quality/04-security-testing.md) |

## Key decisions and design notes

- One chokepoint: all filesystem and process access goes through `agent-sandbox`, enforced by lint rather than convention.
- Open-then-verify, not verify-then-open, because a path can change between the two.
- Untrusted workspaces are read-only with no execute, no network, and no workspace config honored.
- The OS sandbox is an additional layer, never a replacement for the guards — Landlock may be unavailable, and the guards may not be.
- Command matching is on normalized argv shape, so a grant cannot be widened by string tricks.

## Contracts touched

- Path guard API and its error codes
- Command guard API and match semantics
- Sandbox capability report shown in the UI

## Dependencies

- Phase 2 (design)
- Phase 3 (settings and UI surfaces)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Windows containment weaker than Unix | Uneven security posture | Job object plus restricted token plus guards; capability honestly reported; documented in the threat model |
| Landlock unavailable on older kernels | Degraded Linux containment | Detect and report; guards still enforce; policy may require the OS layer for regulated orgs |
| Guard false positives | Legitimate work blocked | Corpus-driven tuning plus clear error messages naming the rule that fired |
| Symlink and case-insensitivity edge cases | Escape | Dedicated corpus, fuzzing, and platform-specific tests |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Canonicalization, glob matching, argv parsing |
| Security | Full traversal and command corpora on all three platforms |
| Integration | Real temp filesystems, real symlinks, real processes |
| Fuzz | Path and command parsers, persisted corpora, no crashes |
| Platform | OS sandbox verifiably blocks a probe tool's forbidden syscall |

## Metrics and targets

| Metric | Target |
|---|---|
| Path guard overhead | < 1 ms per call |
| Command guard overhead | < 1 ms per call |
| Escape suite | 0 escapes on all platforms |
| Orphaned processes after abort | 0 |

## Exit criteria

- [ ] Escape suite passes on macOS, Windows, and Linux with zero escapes
- [ ] Lint proves no filesystem or process access exists outside the sandbox crate
- [ ] OS sandbox verified active, with degradation reported when unavailable
- [ ] Untrusted workspace mode enforces read-only
- [ ] Fuzz targets run in CI with committed corpora
- [ ] Abort leaves no orphaned processes

## Deferred from this phase

- Per-tool policy decisions (Phase 7)
- Egress control during execute (Phase 7 policy, enforced here)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
