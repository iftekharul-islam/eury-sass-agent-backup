# ADR-0002: Tauri over Electron and PySide6

Spec-Version: 1.0.0

**Status:** Accepted  
**Date:** 2026-08-16

## Context

Previous desktop app (`code-old`) used PySide6 + Python agent loop. Limitations: large distribution, GIL for concurrency, markdown fence tool parsing, no native Cersei integration.

Alternatives: Electron + Node agent, Electron + Rust sidecar, pure Rust GUI (egui/iced), VS Code extension.

## Decision

Use **Tauri 2** with **React 19** frontend and **Rust** core crates.

## Consequences

**Positive:**
- Small binary vs Electron; native webview.
- Rust core shares language with Cersei.
- Reuse React/Tailwind patterns from `frontend/`.
- Strong IPC story for streaming events.

**Negative:**
- WebView inconsistencies across platforms (test matrix required).
- Rust learning curve for team.
- Briefcase/Python packaging retired.

**Mitigations:**
- Platform CI matrix (macOS, Windows, Linux).
- Thin Tauri command layer; logic in `crates/`.
