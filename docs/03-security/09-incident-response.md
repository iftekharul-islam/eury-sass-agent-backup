# Incident Response

Spec-Version: 1.2.0

## Severity levels

| Level | Example | Response time |
|-------|---------|---------------|
| SEV1 | Cross-tenant leak, malicious signed release/signing-key compromise, active broad credential theft | 15 min acknowledge, 4 h contain |
| SEV2 | Auth bypass, desktop RCE, confirmed sandbox escape or exploited policy bypass | 1 h acknowledge, 24 h contain |
| SEV3 | Unexploited control weakness, bounded audit gap, low-impact policy error | 4 h acknowledge, 72 h contain/fix plan |
| SEV4 | Minor bug, no security impact | Next sprint |

## Playbooks

### SEV1: Suspected credential leak

1. Revoke all sessions for affected users/orgs.
2. Rotate provider API keys in secret manager.
3. Force desktop re-auth (invalidate refresh tokens).
4. Notify affected enterprise admins within the SEV1 one-hour target; Legal
   owns additional contractual/regulatory deadlines.
5. Post-mortem within 5 business days.

### SEV1: Malicious release or signing-key compromise

1. Pull affected installer from CDN.
2. Revoke the compromised key/version and publish a root-signed deny/keyset
   update through the independent recovery channel.
3. Rebuild from verified source with clean identities; publish only after
   provenance and signature verification.
4. Notify users/admins and require update/removal based on exposure.

### SEV2: Sandbox escape report

1. Reproduce in isolated environment.
2. Disable affected tool via feature flag if confirmed.
3. Patch + expedited release channel.

## Contacts

| Role | Responsibility |
|------|----------------|
| Incident commander | Coordinates response |
| Security lead | Technical assessment |
| Comms | Customer notification |
| Legal | Regulatory obligations |

Each public role maps in the restricted on-call system to a primary, backup,
pager source, authority, and escalation deadline. A missing primary never
blocks the backup from declaring an incident or executing pre-approved
containment.

## Declaration, evidence, and closure

Declare when credible evidence indicates customer/security impact or a Critical
control failure; uncertainty changes scope, not the duty to declare. The
incident record contains id, severity history, commander, primary/backup roles,
affected assets/tenants/regions/versions, timeline, evidence hashes and custody,
decisions/approvers, communications, containment/recovery verification, and
linked corrective actions. Secret/content values are not copied into it.

Closure requires contained exposure, revoked vulnerable credentials/artifacts,
restored monitored service, customer/regulatory decisions recorded, evidence
preserved, residual risk accepted by Security, and every corrective action
owned with a due date. Security runs a tabletop at least twice yearly and after
material trust-boundary changes; signing compromise is exercised annually.

## Customer and administrator communications

The incident commander assigns a communications owner at declaration. Technical mitigation and customer updates run in parallel; an unresolved technical investigation is not a reason to leave affected customers without an update.

| Severity | Initial communication | Update cadence | Closure |
|---|---|---|---|
| SEV1 | Status page and affected enterprise admins within 1 hour; regulatory notification follows contractual/legal timelines | At least every 2 hours while customer impact remains | Customer-facing summary and root-cause analysis within 10 business days |
| SEV2 | Affected admins within 4 hours; public status page when availability or integrity impact is broad | At least every business day until resolved | Summary, remediation, and prevention actions within 15 business days |
| SEV3 | Affected admins when policy, audit, or data obligations are impacted | On material change | Support case update and tracked corrective action |
| SEV4 | Support channel only | As needed | Release note or support resolution |

Messages state the observed impact, affected surface and time window, immediate user action, current mitigation, next-update time, and support contact. They MUST NOT speculate about root cause, expose another customer’s data, include credentials, or promise a completion time before it is evidenced.

Support diagnostic bundles require customer consent and use the redacted export path. The communications owner preserves status-page posts, customer notices, scope decisions, and timestamps with incident evidence.

## Logging for forensics

Retain cloud API access logs 90 days minimum. Audit events per org retention policy (default 1 year enterprise).

## Related documents

- [04-secrets-and-key-management.md](04-secrets-and-key-management.md)
- [../07-ops/06-runbooks.md](../07-ops/06-runbooks.md)
