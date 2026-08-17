# Phase 24 — Enterprise Identity and Governance

Spec-Version: 1.1.0

**Track:** F — Enterprise and GA · **Estimated size:** 2–3 weeks · **Milestone:** —

## Goal

SSO, SCIM, personal access tokens, RBAC surfacing, and cloud policy distribution with signing — all inside the Agent module.

## Why this phase exists here

These are the requirements that gate enterprise deals. They come after the product works, because governance over a moving target is wasted effort.

## In scope

- SAML 2.0 and OIDC login with domain discovery and enforcement
- SCIM 2.0 Users and Groups with complete deprovisioning
- Personal access tokens for headless and air-gapped use
- Agent permission derivation surfaced through `/agent/v1/me`
- Team-scoped policies and the read-only auditor role
- Cloud policy distribution with ETags, signing, and version monotonicity
- Admin pages: policies, identity, devices
- Strict revocation mode for instant access cutoff

## Feature IDs

`F-074`, `F-080`, `F-081`, `F-086`

## Out of scope

- Quotas and budgets (Phase 25)
- SIEM export (Phase 25)

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D24.1 | SAML and OIDC flows bound to the device-code session | [identity](../06-enterprise/01-identity-sso-scim.md) |
| D24.2 | Domain discovery and SSO enforcement with DNS verification | [identity](../06-enterprise/01-identity-sso-scim.md) |
| D24.3 | SCIM endpoints with idempotent creates and full deprovisioning | [identity](../06-enterprise/01-identity-sso-scim.md) |
| D24.4 | Personal access tokens with scopes and revocation | [identity](../06-enterprise/01-identity-sso-scim.md) |
| D24.5 | Permission resolution returned explicitly to the client | [RBAC](../06-enterprise/02-rbac-and-org-model.md) |
| D24.6 | Policy distribution endpoint with ETag, signature, and monotonic versions | [workspace policies](../06-enterprise/03-workspace-policies.md) |
| D24.7 | Admin policy, identity, and device pages | [admin console](../06-enterprise/06-admin-console-spec.md) |
| D24.8 | Strict revocation mode with a configurable offline window | [RBAC](../06-enterprise/02-rbac-and-org-model.md) |

## Key decisions and design notes

- SSO lives in the Agent module with its own session binding; the desktop never renders an IdP form.
- Deprovisioning completes within one request: refresh chains revoked, devices marked revoked, membership deactivated.
- Policies are signed for enterprises, and a lower version is rejected so an attacker cannot replay a permissive policy.
- Client-side permission enforcement is for UX; the server re-checks everything.

## Contracts touched

- SSO discovery response
- SCIM subset per RFC 7644
- Policy distribution response with signature
- Personal access token format and scopes

## Dependencies

- Phase 10 (auth foundation)
- Phase 7 (local policy engine)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| IdP variation | Integration failures | Test matrix across Okta, Entra ID, Google, Keycloak; strict standards compliance |
| Deprovisioning incomplete | Access after offboarding | Single-transaction revocation, audit event, and a residual-access test asserting the 15-minute ceiling |
| Policy signing key handling | Trust failure | HSM storage, rotation planned a release ahead, verification test in the client |
| SCIM surface exposed unintentionally | Attack surface | Routes 404 without a configured token |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Assertion validation, attribute mapping, SCIM patch semantics |
| Integration | Full SSO login per IdP against test tenants |
| Security | Unsigned and replayed assertions rejected; state bound to device code; deprovision revokes everything |
| Contract | SCIM conformance subset; policy distribution ETag behavior |

## Metrics and targets

| Metric | Target |
|---|---|
| SSO login completion | < 45 s median |
| Deprovision to token revocation | < 1 s |
| Residual access after deprovision | ≤ 15 min (0 in strict mode) |
| Policy fetch latency (cached) | < 30 ms p95 |

## Exit criteria

- [ ] SSO works against at least three IdPs
- [ ] SCIM provisioning and deprovisioning verified end to end
- [ ] Policy distribution works with ETags, signing, and downgrade rejection
- [ ] Strict revocation mode cuts access immediately
- [ ] Admin policy, identity, and device pages functional

## Deferred from this phase

- Multi-org membership (open question Q05)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
