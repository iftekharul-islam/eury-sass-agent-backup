# ADR-0004: SQLite Local Store with Encryption

Spec-Version: 1.0.0

**Status:** Accepted  
**Date:** 2026-08-16

## Context

`code-old` used whole-file JSON (`sessions.json`) — slow, corruptible, no query model, plaintext tokens in `auth.json`.

Alternatives: JSON files per entity, RocksDB, embedded PostgreSQL.

## Decision

Use **SQLite** with **encryption at rest** (SQLCipher or platform equivalent) for conversations, runs, settings, audit queue. Keys derived from OS keychain + device-specific salt.

## Consequences

**Positive:**
- ACID, migrations, indexed queries.
- Single file backup/export.
- Industry standard for desktop apps.

**Negative:**
- Migration tooling required.
- Encrypted DB harder to debug (export tool needed).

**Mitigations:**
- `agent-store` migration framework from day one.
- Support export to JSON for support (user-initiated, redacted).
