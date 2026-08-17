# Compatibility and Lifecycle

Spec-Version: 1.0.0

This policy makes compatibility explicit across the desktop application, `/agent/v1` cloud API, NDJSON event protocol, local SQLite schema, and organization policy documents.

## Support windows

| Surface | Supported versions | Enforcement |
|---|---|---|
| Desktop release | Current stable and previous stable minor (`N`, `N-1`) | Release service supplies `minSupported`; unsupported clients receive an update-required state |
| `/agent/v1` HTTP API | Version `v1` through its published deprecation window | Additive fields only within v1; breaking work requires v2 |
| NDJSON event protocol | Current and previous protocol minor | Event `schemaVersion` is negotiated at stream start |
| Local SQLite migrations | Current app can open the previous two schema versions | Forward migrations are transactional and irreversible only after verified backup/checkpoint |
| Organization policy | Current and previous `schemaVersion` | Unknown restrictive fields fail closed; unknown permissive fields do not widen access |

## Compatibility rules

1. Adding an optional field, event type, enum member with a safe unknown-state UI, or endpoint is backward compatible.
2. Removing/renaming fields, changing semantics, lowering a limit, or changing default permission behavior is breaking.
3. A breaking change requires a new API/protocol/schema version, migration notes, fixture updates, and a rollback plan.
4. Desktop clients MUST ignore unknown optional event fields but MUST fail closed for unknown permission decisions, tool classes, policy requirements, or security-critical event types.
5. The gateway MUST retain a model catalog mapping for retired model IDs so historical audits stay interpretable.

## Release and deprecation process

1. Propose the change in an ADR or affected normative spec.
2. Publish compatibility impact, migration path, minimum desktop version, and rollback condition in release notes.
3. Add old/new contract fixtures and an `N-1` upgrade test before rollout.
4. Announce externally visible deprecation at least two stable releases or 90 days before enforcement, whichever is longer, except for an active security incident.
5. During a security emergency, block unsafe versions immediately, publish rationale, and provide a remediation path.

## Required evidence

| Change | Evidence before stable rollout |
|---|---|
| Desktop migration | Fresh install, N-1 upgrade, interruption/restart, rollback, and encrypted-data recovery tests |
| HTTP/event change | Old/new golden fixtures, compatibility test matrix, telemetry for unknown-client errors |
| Policy schema change | Merge/property tests, fail-closed unknown-field test, admin migration preview |
| Provider/model removal | Catalog compatibility mapping, customer notice, audit-read test |
| API deprecation | Usage inventory, customer notice, date, migration guide, support owner |

## Ownership

Platform owns desktop, protocol, and migration compatibility. Backend owns cloud API compatibility. Security owns policy and security-emergency exceptions. Product owns customer communication and deprecation timing. No release can bypass another owner’s required evidence.

## Related documents

- [Cloud API contract](../04-specs/06-cloud-api-contract.md)
- [Event protocol](../04-specs/03-event-protocol-spec.md)
- [Local data model](../04-specs/05-local-data-model.md)
- [Auto-update and rollback](04-auto-update-and-rollback.md)
- [Release management](08-release-management.md)
