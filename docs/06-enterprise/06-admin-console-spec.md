# Admin Console Specification

Spec-Version: 1.1.0

New pages in the existing `frontend/` admin area, all under `/admin/agent/*`. They call `/agent/v1/admin/*` only — never a legacy `code` or `desktop-releases` endpoint ([naming and migration map](../00-overview/05-naming-and-migration-map.md)).

## Routes

| Route | Purpose | Permission |
|---|---|---|
| `/admin/agent` | Overview: active devices, runs today, denied ops, spend, release health | `agent:view_usage` |
| `/admin/agent/releases` | Publish and manage installers | platform staff (`agent:manage_releases`) |
| `/admin/agent/releases/:id` | Release detail, artifacts, checksums, rollout | platform staff |
| `/admin/agent/policies` | Policy list, editor, versions, dry-run | `agent:manage_policies` |
| `/admin/agent/policies/:id` | Version history and diff | `agent:manage_policies` |
| `/admin/agent/audit` | Search, chain verification, export | `agent:view_audit` |
| `/admin/agent/usage` | Usage and budgets | `agent:view_usage` / `agent:manage_budgets` |
| `/admin/agent/devices` | Device inventory and revocation | `agent:manage_devices` |
| `/admin/agent/identity` | SSO and SCIM configuration | `agent:manage_sso_scim` |
| `/admin/agent/mcp` | MCP server registry approvals | `agent:manage_mcp_registry` |

Legacy `/admin/desktop-releases` stays for the deprecated client and is not modified.

## Releases

| Capability | Detail |
|---|---|
| Upload | Per-platform artifact (`darwin-aarch64`, `darwin-x86_64`, `win32-x86_64`, `linux-x86_64`) to `agent-releases/<version>/` |
| Integrity | SHA-256 computed server-side and compared to the CI-supplied value; mismatch blocks publish |
| Manifest signing | Manifest signed with the release key; the signature is stored on `AgentRelease` |
| Metadata | Version, channel, release notes (markdown), `minSupported`, `mandatory` flag |
| Activation | One active release per channel, swapped in a transaction |
| Staged rollout | Percentage per channel, evaluated by a stable hash of `deviceId` |
| Rollback | One click reactivates the previous release and records `admin.release_rolled_back` |
| Health | Update success/failure counts, crash rate by version, adoption curve |
| Guard rails | Cannot activate a release whose artifacts are missing, unsigned, or whose `minSupported` exceeds its own version |

Publishing normally happens from CI ([CI/CD](../07-ops/02-ci-cd-pipelines.md)); the console is for promotion, rollback, and emergencies.

## Policies

Editor with three modes: preset picker, structured form for common fields, and a schema-validated JSON editor. Additional capabilities:

- Diff against the currently active version before activating.
- **Dry-run:** replay the last N runs (metadata only) and report which operations the candidate policy would have blocked.
- Scope selection: org, team, or user.
- Version history with author, timestamp, and one-click rollback.
- Activation invalidates policy ETags so desktops pick up changes at their next poll.

Schema: [workspace policies](03-workspace-policies.md).

## Audit

Search by user, device, run, event type, severity, workspace hash, and date range. Row expansion shows the full envelope. Actions: export CSV/JSON (queued above 50k rows, delivered as an expiring signed link), verify hash chain per device, and configure SIEM delivery. Every export and configuration change is audited.

## Usage and budgets

Org totals, per-seat table, per-model split, denied-request reasons, daily trend, top spenders, CSV export. Budget controls: org monthly budget, per-seat budget, alert thresholds, overage ceiling. Changes take effect within 60 s ([usage, quotas, budgets](05-usage-quotas-and-budgets.md)).

## Devices

Inventory: user, device name, platform, app version, OS version, last seen, policy version in effect, revocation state. Actions: revoke device (invalidates refresh chain immediately), force update by raising `minSupported` for the org, and view that device's recent audit events.

## Identity

SSO configuration (protocol, endpoints, certificate/JWKS, attribute map, enforcement, JIT), domain verification status with the required DNS TXT record, SCIM token issuance and rotation, and group-to-role mapping. Secrets are write-only in the UI: after saving, only a fingerprint is displayed.

## MCP registry

Org-approved MCP servers: name, transport, manifest hash, signature status, tools exposed, approval state, and which policies reference them. Approving a server records the manifest hash so a later manifest change requires re-approval ([MCP integration](../04-specs/10-mcp-integration-spec.md)).

## Cross-cutting requirements

| Concern | Rule |
|---|---|
| Auth | `AgentAdminGuard` inside the Agent module; the console additionally uses the platform's existing admin session for page access |
| Authorization | Server-side permission check on every request; the UI hides nothing that the server would allow, and disables what it would deny with a reason |
| Admin audit | Every mutating action writes an `admin.*` event with actor, target, before/after, and request id |
| Dangerous actions | Typed confirmation for release activation, `minSupported` bumps, policy activation, and mass device revocation |
| Idempotency | Mutating requests accept an `Idempotency-Key` |
| Pagination | Cursor-based, max 200 rows per page |
| i18n | Admin UI is English-only in v1 (internal tool) |
| Accessibility | Same AA bar as the desktop app |
| Empty states | Every page explains what the object is and links to the relevant doc |

## Delivery

Policy, identity, and device pages ship in Phase 24. Audit, usage, and budget pages ship in Phase 25. Release management pages ship in Phase 27 with the release pipeline. The MCP registry page ships in Phase 19 alongside MCP support, with org-level approval wired up in Phase 24.

## Related documents

- [Cloud API contract](../04-specs/06-cloud-api-contract.md)
- [Workspace policies](03-workspace-policies.md)
- [Audit and retention](04-audit-and-retention.md)
- [Packaging, signing, notarization](../07-ops/03-packaging-signing-notarization.md)
