# RBAC and Organization Model

Spec-Version: 1.2.0

## Existing platform roles

Defined in `backend/src/modules/organizations/organization-permissions.ts`.

| Role | Existing permissions |
|---|---|
| `owner` | view, manage_settings, manage_members, manage_files, view_billing |
| `admin` | same as owner |
| `member` | view |

The Agent module **reads** `OrganizationMember.role` and derives Agent permissions from it. It does not modify the platform's permission constants, and it does not import `OrganizationsService` ([backend module structure](../04-specs/16-backend-module-structure.md)).

## Agent permissions

Declared inside the Agent module (`agent/auth/agent-permissions.ts`):

| Permission | owner | admin | member | Meaning |
|---|:--:|:--:|:--:|---|
| `agent:use` | ✓ | ✓ | ✓ | Sign in and run the desktop agent |
| `agent:use_managed_gateway` | ✓ | ✓ | ✓* | Use org-billed inference (*if seat assigned) |
| `agent:use_byok` | ✓ | ✓ | ✓* | Use own provider key (*if policy allows) |
| `agent:manage_policies` | ✓ | ✓ | — | Create/activate org and team policies |
| `agent:view_audit` | ✓ | ✓ | — | Search and export audit events |
| `agent:manage_devices` | ✓ | ✓ | own | Revoke devices (members: only their own) |
| `agent:manage_releases` | platform admin only | — | — | Publish installers |
| `agent:view_usage` | ✓ | ✓ | own | Usage dashboards |
| `agent:manage_budgets` | ✓ | — | — | Set spend caps |
| `agent:manage_sso_scim` | ✓ | ✓ | — | Identity configuration |
| `agent:approve_exceptions` | ✓ | ✓ | — | Grant one-off policy exceptions |
| `agent:manage_mcp_registry` | ✓ | ✓ | — | Approve MCP servers org-wide |

`agent:manage_releases` is a **platform-staff** capability, checked against the existing platform admin flag, not an org role.

### Optional role: `agent_auditor`

Read-only compliance role: `agent:view_audit` + `agent:view_usage`, no `agent:use`. Implemented as an Agent-module role assignment table entry keyed by `(organizationId, userId)` rather than a new platform role, so the platform role enum stays untouched.

## Teams

Reuses the platform `Team` / `TeamMember` models. Teams matter to the Agent for exactly two things:

1. Team-scoped policy overrides (`AgentPolicy.scope = "team"`).
2. Group-to-role mapping from SCIM.

A user in multiple teams receives the **intersection of permissions and the union of restrictions** — the most restrictive result. This makes team membership monotonically safe.

## Effective permission resolution

```
platform role (owner|admin|member)
  → base Agent permission set
  → minus org policy restrictions
  → minus team policy restrictions
  → minus user caps
  → minus package entitlements and quota state
= effective permissions
```

Resolution happens server-side and is returned by `GET /agent/v1/me` as an explicit list, so the desktop never re-derives it from a role string. Policy merge semantics: [permission and policy engine](../03-security/03-permission-and-policy-engine.md).

## Desktop enforcement

| Concern | Rule |
|---|---|
| Fetch | Permissions + policy ETag fetched at login and every 15 min |
| Cache | Stored in SQLite with `fetchedAt`; used offline |
| Offline grace | Default 24 h; after that, managed features stop and BYOK continues only when the cached `byok` entitlement and policy allow it |
| Revocation | `agent:use` removal takes effect at the next `/me` or refresh, ≤ 15 min |
| UI | Missing permissions render as disabled controls with the reason, never hidden silently |
| Fail-closed | Unparseable or missing permission payload disables all write/execute tools |

## Seat model

| Plan | Included seats | Managed gateway | BYOK |
|---|---:|---|---|
| Free | 1 | Limited personal quota | No |
| Starter | 1 | Personal quota | Yes |
| Pro | 1 | Personal quota and priority catalog | Yes |
| Business | 5 | Organization quota and per-seat cap | Yes unless policy denies |
| Enterprise | Contracted | Contracted quota, budgets, optional overage | Policy-controlled |

Seat assignment is read from the existing `Subscription` / seat records. A
member without an assigned seat is denied the managed gateway with
`EURY_ENTITLEMENT_NO_SEAT`; BYOK remains available only when the effective
entitlements and policy independently allow it.

## Audit

Every permission-relevant change emits an event: `rbac.role_changed`, `rbac.seat_assigned`, `rbac.seat_revoked`, `rbac.auditor_granted`, `policy.activated`, `device.revoked`. Admin actions in the console are audited with actor, before/after, and request id ([audit and retention](04-audit-and-retention.md)).

## Delivery

Base permission derivation ships in Phase 10 with authentication. Policy scoping by team, the auditor role, and admin management UI ship in Phase 24.

## Related documents

- [Identity, SSO, SCIM](01-identity-sso-scim.md)
- [Workspace policies](03-workspace-policies.md)
- [Usage quotas and budgets](05-usage-quotas-and-budgets.md)
- [Permission and policy engine](../03-security/03-permission-and-policy-engine.md)
