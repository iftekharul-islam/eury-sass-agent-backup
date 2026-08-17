# ADR-0007: Rust Workspace Crate Split

Spec-Version: 1.0.0

**Status:** Accepted  
**Date:** 2026-08-16

## Context

Monolithic `src-tauri` becomes unmaintainable at enterprise scale. Need clear boundaries for testing, compile times, and ownership.

## Decision

Split Rust into crates:

- `agent-types` — shared types, no I/O
- `agent-sandbox` — path/command guards
- `agent-policy` — policy merge + decisions
- `agent-store` — SQLite
- `agent-index` — search/index
- `agent-memory` — graph + EURY.md
- `agent-tools` — tool implementations
- `agent-core` — engine trait + orchestration + cersei_adapter module

`apps/desktop/src-tauri` contains only Tauri commands and wiring.

## Consequences

**Positive:**
- Parallel development, focused unit tests.
- Faster incremental builds for crate-only changes.

**Negative:**
- Workspace coordination overhead.
- Public API between crates must be stable.

**Mitigations:**
- `agent-types` as single source of truth for IPC JSON.
