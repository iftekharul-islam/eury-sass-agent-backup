# Privacy and Data Residency

Spec-Version: 1.2.0

## Default posture

**Local-first, not cloud-never.** Agent tools execute locally. Content crosses a
boundary only for the selected model route or an explicitly enabled service,
after purpose, policy, minimization, residency, retention, and redaction checks.

## Data classification

| Data | Stored locally | Sent to cloud | Sent to LLM |
|------|----------------|---------------|-------------|
| Source files | Yes | Retrieved chunks transit Eury only on managed route; not persisted by default | Only selected chunks |
| Full repo tree | Index metadata | No | No |
| Chat messages | Yes | If sync enabled | Yes (in prompt) |
| Tool results | Yes (run journal) | Managed model context transits gateway; audit is metadata-only unless explicit full-payload policy | Selected/truncated results |
| API keys (BYOK) | Keychain | Never | Via provider only |
| Email, billing | No | Yes | No |
| Telemetry | Local queue up to 24 h | Off/user-choice/org-required per policy; content prohibited | No |
| Image attachments | Encrypted attachment store | Bytes/reference may transit managed attachment/model route; persistence follows declared retention | Only after capability, policy, residency, and metadata checks |
| Generated images | Encrypted attachment store | Provider request/result only when enabled | Image prompt and approved reference inputs only |

## User controls

| Setting | Default | Description |
|---------|---------|-------------|
| Cloud conversation sync | Off | Opt-in per account |
| Crash reports | Off | Opt-in |
| Usage analytics | Off / user choice | Enterprise may require disclosed metadata-only analytics; air-gapped always off |
| Audit upload | On (enterprise) | Org-controlled |

## LLM providers

BYOK: user's DPA with provider applies.

Managed: Eury is a transit processor to an approved regional provider; the
subprocessor, purpose and residency, region, zero retention agreements where available,
and provider-deletion capability are resolved before transfer. Gateway operational logs
are metadata-only.

## Data residency (enterprise)

Phase 24+ deploys region-specific gateway, database, object, telemetry, audit,
log, and backup routes. Before then, an organization requiring an unsupported
region receives `EURY_RESIDENCY_UNAVAILABLE`; the request fails closed and is
not silently routed elsewhere. Provider/model, sync, audit, telemetry,
attachments, backups, support export, and subprocessors all use the same
effective residency decision.

## Purpose and retention rule

Each outbound record carries `{ purpose, dataClasses, route, region,
retentionPolicyId, policyVersion }`. Purpose values are model inference, sync,
audit, telemetry, billing/identity, or support export. Re-use for another
purpose is forbidden. Full-payload audit is a distinct, explicitly disclosed
policy that can replicate source content and therefore requires residency,
retention, access, export, and deletion controls equal to conversation content.

## Deletion

| Request | Action |
|---------|--------|
| Delete account | Revoke tokens; delete cloud data per retention policy |
| Delete local data | Settings → Clear all data; wipe app data dir |
| GDPR export | API export conversations, audit, billing |

## Related documents

- [04-secrets-and-key-management.md](04-secrets-and-key-management.md)
- [08-compliance-baseline.md](08-compliance-baseline.md)
- [Multimodal and attachments](../04-specs/17-multimodal-and-attachment-spec.md)
- [Provider and model governance](../02-architecture/07-provider-and-model-governance.md)
