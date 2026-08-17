# ADR-0006: Deny-by-Default Permissions

Spec-Version: 2.0.0

**Status:** Accepted  
**Date:** 2026-08-16

## Context

Coding agents with shell and write access are high-risk. `code-old` used opt-in approval for some tools but `AllowAll`-equivalent defaults in dev. Enterprise customers require provable controls.

## Decision

“Deny by default” means **no implicit privileged allow**, not that every
ordinary request receives the non-overridable `deny` decision.

- `read` is allowed only inside a trusted workspace and applicable mode/policy.
- Ordinary `write`, `execute`, `network`, and `mcp` requests default to
  `needsApproval`.
- `write_outside_workspace` additionally requires a user-selected capability
  root and exact-shape approval.
- Forbidden operations receive non-overridable `deny` and never produce an
  approval prompt.

The canonical decision enum is `allow | needsApproval | deny`.
`GrantScope` is `once | run | session | always`; `deny` is not a scope.
Cersei `InteractivePolicy` bridges `needsApproval` to the UI.

Session/standing grants are explicit, shape-bound, revocable, and capped by the
effective policy. A grant never widens mode, entitlement, capability root,
egress, or organization policy.

## Consequences

**Positive:**
- Security-first posture for enterprise sales.
- Aligns with Cersei permission levels and bash classifier.

**Negative:**
- More clicks for power users.

**Mitigations:**
- Scoped "always allow" for trusted workspaces.
- Org policy can pre-approve read-only sets.
