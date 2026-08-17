# Permission and Policy Engine

Spec-Version: 2.0.0

**Owner:** Security · **Lifecycle:** approved design contract

## Policy sources and decision overlays

The canonical schema, file path, source order, and merge algebra are defined in
[workspace policies](../06-enterprise/03-workspace-policies.md):

1. Product defaults.
2. Organization policy.
3. Team policy.
4. User caps.
5. Workspace policy at `.eury/policy.json`.
6. Local settings.

Mode is a registry filter applied before policy evaluation. Session grants and
per-call approvals are decision overlays applied after the merged policy; they
can satisfy `needsApproval` but can never override `deny` or widen the tool set.

Canonical machine-readable fields are defined by
[`workspace-policy.schema.json`](../04-specs/schemas/workspace-policy.schema.json)
and [`security-types.schema.json`](../04-specs/schemas/security-types.schema.json).
Consumers MUST reject unknown security-relevant enum values and fields.

## Cersei integration

```rust
InteractivePolicy::new(|req| {
    policy_engine.evaluate(req).await
})
```

Map `allow` directly, `deny` directly, and suspend `needsApproval` through
Cersei's interactive callback until the exact approval id resolves or expires.
Timeout, cancellation, disconnect, or malformed response resolves to `deny`.

## Permission levels (align with Cersei)

| Level | Tools | Default |
|-------|-------|---------|
| None | mode switches and app-owned plan/task state | Allow after schema validation |
| ReadOnly | read, grep, glob, approved read-only MCP | Allow in a trusted workspace |
| Network | web search/fetch, image generation | `needsApproval`; never implicitly allowed |
| Write | write, edit, delete | `needsApproval` |
| Execute | command, package manager, git commit | `needsApproval` |
| Dangerous | destructive/forbidden operations | `deny`; non-overridable |

`defaultDecision: "needsApproval"` is distinct from `deny`. Product defaults
use `needsApproval` for ordinary write/execute/network actions; `deny` is
reserved for prohibitions no lower source or grant may widen.

## Approval UX contract

When approval required, emit `approval_required` event:

```json
{
  "type": "approval_required",
  "approval_id": "uuid",
  "tool_name": "write_file",
  "risk": "write",
  "summary": "Write src/main.rs (142 lines changed)",
  "diff_preview": "...",
  "expires_at": "ISO8601"
}
```

User responds via IPC `approval_respond { approval_id, decision, scope }`.

`PolicyDecision` enum: `"allow" | "needsApproval" | "deny"`.
`ApprovalResponse` decision enum: `allow` | `deny`.

`GrantScope` enum: `"session" | "oneTime" | "workspace"`. `deny` is a decision,
never a grant scope.

`workspace` scope is scoped to the normalized workspace and tool/policy shape; it never bypasses an org policy and is unavailable for critical-risk actions. Network tools use the same scopes, but an organization may require `oneTime` for image generation. Policy sources include `.eury/policy.json` (or cloud `workspace_policy.json`).

## Evaluation order

1. Validate request schema and selected mode.
2. Require feature entitlement and advertised tool/capability.
3. Load and verify product/org/team/user/workspace/local policy sources.
4. Merge using restrictive algebra and classify risk.
5. Verify workspace trust, sandbox capability, path root, model/MCP fingerprint,
   requested egress, and plan-step scope.
6. Apply a matching unexpired grant only to `needsApproval`.
7. Return `allow`, `needsApproval`, or non-overridable `deny`.

| State | Read | Write/execute/network/MCP/outside root |
|---|---|---|
| Missing/unparseable policy payload | Only non-workspace Chat/history; no workspace read | deny |
| Invalid signature or version rollback | Existing verified cache may apply within its contract | deny and emit critical event |
| Cache stale below configured hard limit | Effective cached policy | policy result |
| Cache beyond a required hard limit | Read may remain if org policy allows | deny |
| Sandbox required capability unavailable | Guarded read only if profile permits | deny |
| Approval service/UI unavailable | Existing matching grants only | `needsApproval` resolves to deny |

Policy fields referenced by telemetry, MCP, privacy, and runtime specs MUST
exist in the canonical schema; prose-only aliases such as `block_web_fetch` are
invalid.

## Organization policy schema

`WorkspacePolicy` in
[workspace policies](../06-enterprise/03-workspace-policies.md) is the only
normative schema. Cloud JSON uses those camelCase field names; alternate
snake-case aliases are not accepted.

## Audit

Every policy decision logged locally:

```json
{
  "timestamp": "...",
  "run_id": "...",
  "tool_name": "...",
  "decision": "allow|needsApproval|deny",
  "decision_reason": "policy.default_needs_approval",
  "policy_sources": ["product", "org"],
  "grant_scope": null,
  "policy_version": 7,
  "normalized_shape_hash": "sha256",
  "user_id": "..."
}
```

## Related documents

- [02-sandbox-model.md](02-sandbox-model.md)
- [../04-specs/02-tool-catalog-spec.md](../04-specs/02-tool-catalog-spec.md)
- [../06-enterprise/03-workspace-policies.md](../06-enterprise/03-workspace-policies.md)
