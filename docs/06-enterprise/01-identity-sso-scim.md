# Identity, SSO, and SCIM

Spec-Version: 1.1.0

All identity endpoints for Eury Agent live inside the Agent module under `/agent/v1/*`. The legacy `/auth/ide/*` flow and `IdeAuthSession` table are untouched ([backend module structure](../04-specs/16-backend-module-structure.md)).

## Login paths

| Path | Who | Flow |
|---|---|---|
| Device flow (default) | Individuals, Pro, Business | PKCE device flow → browser approve → tokens in keychain |
| SSO device flow | Enterprise with SSO enforced | Same device flow; the browser step lands on the org's IdP first |
| Personal access token | CI, headless, air-gapped | Scoped, revocable token pasted into settings or provided by env (Phase 24) |

The desktop never renders an IdP form itself and never handles a password. It opens the system browser and waits on `POST /agent/v1/auth/device/poll`.

## Device flow (recap)

Sequence, token lifetimes, and PKCE rules: [cloud architecture](../02-architecture/03-cloud-architecture.md) and [cloud API contract](../04-specs/06-cloud-api-contract.md).

## SSO

### Supported

| Protocol | IdPs validated |
|---|---|
| SAML 2.0 (SP-initiated) | Okta, Azure AD / Entra ID, Google Workspace, OneLogin |
| OIDC (authorization code + PKCE) | Okta, Entra ID, Auth0, Keycloak |

### Domain discovery

1. Desktop sends `POST /agent/v1/auth/device/start` with an optional `email_hint`.
2. Browser page calls `GET /agent/v1/auth/sso/discover?domain=company.com`.
3. Response: `{ "ssoEnabled": true, "provider": "saml", "organizationId": "…", "enforced": true }`.
4. When `enforced`, the approve page immediately redirects to the IdP; local password login is refused for that domain.

Discovery is public but returns only booleans and a display name — never IdP secrets or member lists.

### Configuration (per organization)

| Field | Notes |
|---|---|
| `protocol` | `saml` \| `oidc` |
| `entityId` / `clientId` | SP identifiers |
| `ssoUrl` / `issuer` | IdP endpoints |
| `x509Cert` / `jwksUri` | Signature verification material |
| `enforced` | Blocks non-SSO login for verified domains |
| `jitProvisioning` | Create user on first successful assertion |
| `defaultRole` | Role for JIT users, default `member` |
| `domains[]` | DNS-TXT verified domains |
| `attributeMap` | `email`, `firstName`, `lastName`, `groups` |

Stored in the platform's existing org settings; the Agent module reads it read-only by `organizationId`. Assertion signature validation is mandatory; unsigned assertions are rejected. `RelayState`/`state` is bound to the device code so an assertion cannot be replayed against a different device session.

### Security requirements

| Requirement | Rule |
|---|---|
| Assertion replay | `InResponseTo` + one-time `state` bound to the device code, 10 min window |
| Clock skew | ±2 minutes tolerated |
| Encryption | TLS required; SAML assertions may be encrypted, and must be for `enforced` orgs |
| Domain proof | DNS TXT verification before a domain can enforce SSO |
| Session ceiling | Enterprise may cap refresh-token TTL below 30 d |
| SSO logout | IdP-initiated SLO revokes all Agent refresh tokens for that user |

## SCIM 2.0

Base: `/agent/v1/scim/v2`. Guarded by `AgentScimGuard` with a per-org bearer token (`AGENT_SCIM_TOKEN` for single-tenant, hashed per-org token for multi-tenant). Returns `404` when unconfigured so the surface does not exist for non-enterprise deployments.

| Endpoint | Methods | Behavior |
|---|---|---|
| `/Users` | `GET`, `POST` | List with `filter=userName eq "…"`; create maps to `User` + `OrganizationMember` |
| `/Users/:id` | `GET`, `PUT`, `PATCH`, `DELETE` | `PATCH active:false` and `DELETE` both deprovision |
| `/Groups` | `GET`, `POST` | Maps to `Team` |
| `/Groups/:id` | `GET`, `PUT`, `PATCH`, `DELETE` | Membership changes recompute effective policy |
| `/ServiceProviderConfig` | `GET` | Advertises supported filters and patch ops |
| `/ResourceTypes`, `/Schemas` | `GET` | Static |

| Concern | Rule |
|---|---|
| Pagination | `startIndex` / `count`, max 200 |
| Idempotency | `externalId` unique per org; re-`POST` returns `409` with the existing id |
| Errors | SCIM error schema with `scimType`, mapped from the `EURY_*` code |
| Rate limit | 20 req/s per org |
| Group → role | Group-to-role mapping table per org; unmapped groups grant nothing |

### Deprovisioning (must be complete)

On `active:false` or `DELETE`, within one request:

1. Revoke every `AgentRefreshToken` for the user (`revokedReason = "deprovisioned"`).
2. Mark every `AgentDevice` `revokedAt`.
3. Set `OrganizationMember` inactive.
4. Emit `identity.deprovisioned` audit event.

Access tokens are not individually revocable, so the maximum residual access is the 15-minute access-token TTL. Enterprises requiring instant cutoff enable **strict revocation**: the desktop revalidates with `GET /agent/v1/me` before each run and fails closed when offline for more than a configured window.

## Desktop UX

| State | Behavior |
|---|---|
| First launch | "Sign in to Eury" → device flow; "Use your own API key" available without login for BYOK-only |
| SSO org detected | Button reads "Sign in with your organization" |
| SSO enforced, personal account signed in | Blocking notice with a switch-account action |
| Token expired offline | Grace window per policy (default 24 h) for BYOK; managed gateway blocked immediately |
| Deprovisioned | Next run shows "Access revoked by your organization"; local data retained unless policy demands wipe |
| Device limit reached | Lists existing devices with revoke option |

## Multi-organization

v1 keeps single-org-per-user, inherited from the platform model. The token payload carries one `organizationId`, and policy resolution assumes it. Multi-org membership is tracked as an open question ([Q05](../09-roadmap/open-questions.md)); the API shape (`organizationId` in `/me` rather than implicit) is already forward-compatible.

## Audit events

`auth.device_start`, `auth.device_approved`, `auth.device_denied`, `auth.refresh_rotated`, `auth.refresh_reuse`, `auth.logout`, `sso.assertion_accepted`, `sso.assertion_rejected`, `scim.user_created`, `scim.user_updated`, `identity.deprovisioned`, `device.revoked`.

## Delivery

The Agent auth module, device flow, refresh rotation, and device enrollment ship in Phase 10. SSO discovery, SAML/OIDC, SCIM, personal access tokens, and strict revocation ship in Phase 24.

## Related documents

- [RBAC and org model](02-rbac-and-org-model.md)
- [Cloud API contract](../04-specs/06-cloud-api-contract.md)
- [Secrets and key management](../03-security/04-secrets-and-key-management.md)
- [Naming and migration map](../00-overview/05-naming-and-migration-map.md)
