# Phase 14 — Git

Spec-Version: 1.1.0

**Track:** C — Product surfaces · **Estimated size:** 1–2 weeks · **Milestone:** —

## Goal

First-class git awareness: status, diff, staging, commit, branch context, and agent-authored commits with human-reviewable messages.

## Why this phase exists here

Git is the user's real safety net and the agent's most useful context source. It also makes checkpoints in Phase 18 cheaper by giving them a baseline to reason about.

## In scope

- `git2`-based status, branch, remote, and diff reading
- Staging, unstaging, and committing from the UI
- Agent-facing git tools: status, diff, log, blame, branch — read-only by default
- Commit message generation with a mandatory human review step
- Push and pull as elevated, approval-gated operations
- Branch and worktree awareness for background runs in Phase 21
- Conflict detection with a clear read-only presentation
- `.gitignore` awareness in indexing and explorer

## Feature IDs

`F-047`, `F-055`

## Out of scope

- Interactive rebase and merge resolution UI
- GitHub/GitLab API integration (post-GA)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D14.1 | Git status and diff reading with efficient refresh on file change | [feature catalog](../01-product/02-feature-catalog.md) |
| D14.2 | Stage, unstage, and commit UI with diff review | [tool activity UX](../05-ui/04-tool-activity-and-diff-ux.md) |
| D14.3 | Read-only git tools for the agent | [tool catalog](../04-specs/02-tool-catalog-spec.md) |
| D14.4 | Commit message drafting with required human confirmation | [approval UX](../05-ui/05-approval-and-trust-ux.md) |
| D14.5 | Push/pull as elevated operations requiring explicit approval | [policy engine](../03-security/03-permission-and-policy-engine.md) |
| D14.6 | Conflict state detection and presentation | [feature catalog](../01-product/02-feature-catalog.md) |
| D14.7 | `.gitignore` respected by index, explorer, and search | [indexing spec](../04-specs/09-indexing-and-retrieval-spec.md) |

## Key decisions and design notes

- Git write operations that leave the machine (push) are always elevated risk and never covered by a broad standing grant.
- The agent may draft a commit message but never commits without human confirmation in v1.
- History rewriting operations are not exposed to the agent at all.
- Git is read via `git2` rather than shelling out, so status refresh does not depend on command approval.

## Contracts touched

- Git tool schemas and result shapes
- Git status event for UI decorations

## Dependencies

- Phase 13 (editor and explorer decorations)
- Phase 7 (approval classes)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Destructive git operations | Lost work | Reset, force push, and history rewrite are not agent-accessible; push is approval-gated |
| Status refresh cost on large repos | UI lag | Debounced, incremental refresh; watcher-driven rather than polling |
| Submodules and worktrees | Incorrect status | Explicitly detected and reported; unsupported cases surfaced rather than guessed |
| Credential handling | Leaked credentials | Delegate to the system credential helper; never store git credentials ourselves |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Status parsing, diff generation, ignore handling |
| Integration | Fixture repos: clean, dirty, staged, conflicted, detached HEAD, submodule |
| Security | Push requires approval; no history-rewriting tool is registered |
| Performance | Status refresh on a 50k-file repo |

## Metrics and targets

| Metric | Target |
|---|---|
| Status refresh (50k files) | < 500 ms p95 |
| Diff for a staged change | < 100 ms p95 |
| Destructive git operations reachable by the agent | 0 |

## Exit criteria

- [ ] Status, diff, stage, and commit work from the UI
- [ ] Agent git tools are read-only and pass the tool security suite
- [ ] Commit messages require human confirmation
- [ ] Push and pull are approval-gated as elevated operations
- [ ] Conflicts are detected and clearly presented

## Deferred from this phase

- Merge conflict resolution UI (post-GA)
- Forge API integration (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
