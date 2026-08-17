# Pricing and Packaging

Spec-Version: 2.0.0

Commercial prices and quota values are configuration, not compile-time product behavior. Stable entitlement keys are the contract shared by the catalog, `/agent/v1/me`, gateway guards, policy, and UI. Current seeded prices/limits come from `backend/prisma/seed.ts`; changing a price does not rename an entitlement.

## Entitlement contract

```typescript
interface AgentEntitlements {
  keys: string[];                      // stable registry below
  limits: {
    dailyManagedRuns?: number;         // managed runs per UTC day
    monthlyTokens?: number;            // managed prompt + completion tokens
    concurrentRuns?: number;           // active managed runs per user
    seats?: number;                    // assigned org-managed Agent seats
  };
  effectiveAt: string;
  sourcePlan: "free" | "starter" | "pro" | "business" | "enterprise";
  version: number;
}
```

Unknown keys are ignored by old clients and treated as unavailable. Missing keys deny the capability. The desktop never infers access from a display plan name.

## Stable entitlement registry

| Family | Keys | Meaning |
|---|---|---|
| Base desktop | `agent.desktop`, `agent.updates`, `agent.offline_grace`, `billing.portal`, `models.catalog` | Desktop access, signed updates, offline auth grace, billing link, model catalog |
| Modes/workflows | `modes.ask`, `modes.plan`, `modes.agent`, `modes.build`, `workflows.review` | Permission-profile or workflow availability |
| Tool policy | `tools.workspace_read`, `tools.workspace_write`, `tools.execute`, `tools.network`, `tools.mcp`, `tools.git`, `tools.approvals` | Tool may be advertised after mode, trust, and policy checks |
| Multimodal | `multimodal.vision`, `multimodal.generate` | Vision input and image generation |
| IDE | `ide.explorer`, `ide.editor`, `ide.terminal`, `ide.preview`, `ide.git` | Local desktop surfaces |
| Intelligence | `intelligence.index`, `intelligence.semantic`, `intelligence.memory`, `intelligence.skills`, `agent.checkpoints` | Local intelligence and recovery features |
| Cloud | `managed.inference`, `byok`, `cloud.conversation_sync`, `cloud.projects`, `cloud.personalization` | Model routing and optional cloud state |
| Enterprise | `enterprise.org_admin`, `enterprise.sso`, `enterprise.scim`, `enterprise.policy`, `enterprise.audit`, `enterprise.budgets`, `enterprise.air_gapped` | Organization governance |

An entitlement grants availability, not permission. For example, `tools.execute` only lets the mode/policy engine consider an execute tool; workspace trust, organization policy, sandbox support, and approval still apply.

## Tiers

| Plan | Current configured price (BDT/mo) | Package contract |
|---|---|---|
| **Free** | ৳0 | Desktop, all five safety modes, local read/write approvals, limited managed inference, 20 managed runs/day |
| **Starter** | ৳499 | Free + BYOK and local IDE surfaces, 200 managed runs/day |
| **Pro** | ৳999 | Starter + checkpoints, advanced intelligence/multimodal capabilities, priority catalog, 1000 managed runs/day |
| **Business** | ৳2999 | Pro + 5 managed seats, org administration, policy/budget capabilities as their phases ship, 5000 managed runs/day pooled by contract |
| **Enterprise** | Custom | Business + SSO, SCIM, audit, air-gapped/self-hosted options, contracted quotas, residency, support, and SLA |

Prices are snapshots, not promises in code. Product/Billing may change them
without a desktop release. Legacy seed field `dailyMessages` maps at the Agent
module boundary to the canonical `dailyManagedRuns`; it is not enforced today,
and the gateway MUST enforce it before managed inference is generally available.

## Package bundles

| Capability | Free | Starter | Pro | Business | Enterprise |
|---|---:|---:|---:|---:|---:|
| `agent.desktop`, `agent.updates`, `billing.portal`, `models.catalog` | yes | yes | yes | yes | yes |
| All five `modes.*` entitlements and local read/write approvals | yes | yes | yes | yes | yes |
| `managed.inference` | limited | yes | yes | yes | contracted |
| `byok`, IDE explorer/editor/terminal/git | no | yes | yes | yes | policy-controlled |
| Checkpoints, semantic/memory, vision/generation | no | no | yes | yes | policy-controlled |
| `workflows.review`, MCP, cloud projects/sync | no | no | when shipped | yes | policy-controlled |
| Org admin, policy, budgets | no | no | no | when shipped | yes |
| SSO, SCIM, central audit, air-gapped | no | no | no | no | yes |

“When shipped” means the entitlement may exist in package configuration before the implementation lifecycle reaches `verified`; the client MUST also require the capability/catalog flag.

## Desktop app access

| Plan | Download | Managed inference |
|---|---|---|
| Free | Signed public installer | Limited catalog and quota |
| Paid | Signed public installer | Per-plan quota |
| Enterprise | Signed installer plus contracted MDM/offline channels | Contract limits and policy |

## BYOK

| Plan | BYOK |
|------|------|
| Free | No |
| Starter+ | Yes |

BYOK usage is not counted against `dailyManagedRuns` but remains subject to
local cost caps and abuse controls.

BYOK is still gated by model allowlists, workspace policy, and local cost caps. Provider billing is between the user/customer and provider; Eury displays estimated cost but does not count it as managed usage.

## Seat model (Business+)

Maps to `Subscription.seatCount` and `OrganizationMember`. Agent desktop login counts as seat when org-managed.

A seat is consumed only when assigned to an active organization member, not per device. Removing a seat revokes managed gateway and enterprise entitlements at the next policy/auth refresh; local data remains readable under the user’s retention rights.

## Add-ons (future)

- Extra managed tokens pack
- Dedicated model endpoint
- Extended audit retention (1y → 7y)

## Upgrade paths

In-app: Settings → Billing → opens web checkout (`POST /billing/checkout`).

## Upgrade and downgrade semantics

| Event | Required behavior |
|---|---|
| Upgrade | New signed entitlement version applies on refresh; in-flight run keeps its starting limits |
| Downgrade | No local data deletion; unavailable features become read/export-only where possible |
| Quota reduction | Existing run completes; new managed calls use the lower limit |
| Seat removal | Managed/enterprise access revoked; refresh tokens for other personal access follow account policy |
| Entitlement service unavailable | Use the last signed cached entitlement during offline grace; never grant a key absent from cache |
| Refund/cancellation | Access remains until the billing effective date; audit records retain their contractual policy |

## Commercial open decisions

Overage billing, enterprise retention, managed image retention, and support/SLA values remain owned questions in [open questions](../09-roadmap/open-questions.md). No implementation may invent defaults to unblock delivery.

## Related documents

- [01-personas-and-jobs.md](01-personas-and-jobs.md)
- [../06-enterprise/05-usage-quotas-and-budgets.md](../06-enterprise/05-usage-quotas-and-budgets.md)
