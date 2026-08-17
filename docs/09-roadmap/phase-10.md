# Phase 10 — Identity

Spec-Version: 1.1.0

**Track:** C — Product surfaces · **Estimated size:** 2 weeks · **Milestone:** —

## Goal

Ship the Agent-owned authentication stack: a self-contained NestJS module at `/agent/v1/auth/*`, PKCE device flow, rotating refresh tokens, and keychain storage on the desktop.

## Why this phase exists here

Everything commercial and enterprise depends on identity, and it must be built as an isolated module from the start. Reusing the legacy `/auth/ide/*` flow would couple the new product to the deprecated one and inherit its open-endpoint bug.

## In scope

- `AgentModule` created in the backend with its own controllers, guards, DTOs, and config
- PKCE S256 device flow: start, browser exchange, poll
- Rotating refresh tokens with chain revocation on reuse detection
- `AgentAuthGuard` with no fallback and boot-time config validation
- Prisma migration `agent_identity`: `AgentDevice`, `AgentAuthSession`, `AgentRefreshToken`
- Device enrollment with an Ed25519 signing key for audit batches
- Web authorize page at `/{locale}/agent/authorize`
- Desktop login UI, session state, token refresh, logout, device management
- Module isolation lint check in CI

## Feature IDs

`F-010`, `F-011`, `F-012`, `F-013`, `F-014`, `F-072`, `F-073`

## Out of scope

- SSO and SCIM (Phase 24)
- Managed gateway (Phase 11)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D10.1 | `AgentModule` skeleton with all sub-areas and zero feature-module imports | [module structure](../04-specs/16-backend-module-structure.md) |
| D10.2 | Device flow endpoints with rate limiting and slow-down handling | [cloud API](../04-specs/06-cloud-api-contract.md) |
| D10.3 | Refresh rotation with reuse detection and chain revocation | [cloud architecture](../02-architecture/03-cloud-architecture.md) |
| D10.4 | `agent_identity` Prisma migration, additive only | [cloud data model](../04-specs/07-cloud-data-model.md) |
| D10.5 | `GET /agent/v1/me` returning explicit permissions and limits | [RBAC](../06-enterprise/02-rbac-and-org-model.md) |
| D10.6 | Device enrollment and key registration | [audit and retention](../06-enterprise/04-audit-and-retention.md) |
| D10.7 | Desktop keychain storage for access and refresh tokens | [secrets](../03-security/04-secrets-and-key-management.md) |
| D10.8 | Web authorize page with clear device identification | [identity](../06-enterprise/01-identity-sso-scim.md) |
| D10.9 | CI isolation check proving no cross-module imports | [CI/CD](../07-ops/02-ci-cd-pipelines.md) |

## Key decisions and design notes

- The Agent module owns its tokens, tables, and guards; it duplicates a little logic rather than coupling to the auth module.
- PKCE `plain` is rejected outright; only S256 is accepted.
- Refresh reuse is treated as compromise: the whole chain is revoked and an audit event is raised.
- Missing auth configuration fails app boot. There is no environment variable that can make an Agent route public.
- BYOK users can work without signing in at all, so login is never a hostage for basic functionality.

## Contracts touched

- `/agent/v1/auth/*` request and response shapes
- `AgentPrincipal` shape
- `AgentDevice`, `AgentAuthSession`, `AgentRefreshToken` schemas

## Dependencies

- Phase 3 (settings UI)
- Phase 9 (local session state)
- Phase 2 (secret rules)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Token theft from a compromised machine | Account access | Short access TTL, rotation, device binding, revocation, and strict-revocation mode for enterprises |
| Device flow abuse | Phishing a user into approving | Device name and platform shown prominently; short TTL; single-use codes; rate limits |
| Accidental coupling to legacy auth | Isolation broken | CI import check plus review ownership on the module path |
| Keychain unavailable | Cannot store tokens | Explicit error with a documented remedy; never fall back to a file |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | PKCE verification, rotation, reuse detection, guard behavior |
| Contract | Every auth endpoint against golden fixtures |
| Negative | No token, expired, wrong audience, `alg:none`, revoked device, missing config |
| Integration | Full desktop login against a local backend, including logout and refresh |
| Security | Rate-limit enforcement; replayed device code and assertion rejection |

## Metrics and targets

| Metric | Target |
|---|---|
| Login flow completion (user-paced) | < 30 s median |
| `/agent/v1/auth/refresh` latency | < 50 ms p95 |
| Refresh reuse detection | 100% in tests |
| Cross-module imports in the Agent module | 0 |

## Exit criteria

- [ ] Desktop login works end to end against a local backend
- [ ] Tokens are stored only in the OS keychain
- [ ] Refresh rotation and reuse detection verified
- [ ] Negative-auth suite green, including the missing-config boot failure
- [ ] Prisma migration is additive and reversible by table drop
- [ ] CI isolation check passes

## Deferred from this phase

- SSO, SCIM, and personal access tokens (Phase 24)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
