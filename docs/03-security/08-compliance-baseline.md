# Compliance Baseline

Spec-Version: 1.0.0

## Target

SOC 2 Type II readiness for Eury Cloud control plane. Desktop app supports customer compliance evidence (audit export, policy enforcement).

## Control mapping (summary)

| SOC 2 criteria | Implementation |
|----------------|----------------|
| CC6.1 Logical access | JWT + refresh; SSO (Phase 24); RBAC |
| CC6.6 Encryption | TLS in transit; SQLite encryption at rest; keychain |
| CC7.2 Monitoring | Audit logs; metrics; alerting (Phase 26) |
| CC8.1 Change management | CI/CD; signed releases; ADRs |
| CC9.2 Risk mitigation | Threat model; sandbox; penetration test (Phase 28) |

## Audit evidence

Enterprise customers receive:

- Export of audit events (CSV/JSON)
- Policy version history
- User access reports (org members, last login)

## Desktop compliance features

- Deny-by-default tools
- Configurable data collection (telemetry off by default)
- Local-only mode documentation for regulated environments

## Vendor management

| Vendor | Data shared | Review cadence |
|--------|-------------|----------------|
| LLM providers | Prompts (managed/BYOK) | Annual |
| Cersei (OSS) | None (library) | Per release |
| Tavily (web search) | Search queries | Annual |

## Penetration testing

Annual external pentest before enterprise GA; remediate critical within 30 days.

## Related documents

- [../06-enterprise/04-audit-and-retention.md](../06-enterprise/04-audit-and-retention.md)
- [09-incident-response.md](09-incident-response.md)
