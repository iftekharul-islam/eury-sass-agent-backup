# Competitive Landscape

Spec-Version: 1.1.0

High-level comparison informing product and architecture decisions. Numbers are indicative order-of-magnitude, not benchmarks run by Eury.

## Category

| Product | Form factor | Embeddable SDK | Our positioning |
|---------|-------------|----------------|-----------------|
| **Cursor** | Desktop IDE (Electron) | No | Match UX; beat on local latency + enterprise policy |
| **Claude Code** | CLI | No | Match agent depth; desktop GUI + audit |
| **Windsurf** | Desktop IDE | No | Match flow state; open engine boundary (Cersei) |
| **GitHub Copilot Workspace** | Cloud + IDE plugins | Partial | We stay local-first; optional sync |
| **Cersei / Abstract CLI** | Library + CLI | Yes | We build product on Cersei, not compete with SDK |

## Capability matrix

| Capability | Cursor | Claude Code | Eury Agent (target) |
|------------|--------|-------------|---------------------|
| Local file tools | Yes | Yes | Yes |
| Terminal | Yes | Yes | Yes (visible PTY) |
| Multi-agent | Limited | Subagents | Planner → implementer → tester → reviewer |
| Graph memory | No | File + LLM recall | Cersei graph (~100µs recall) |
| MCP | Yes | Yes | Yes (Phase 19) |
| Enterprise SSO | Yes | Anthropic org | SAML/OIDC + SCIM (Phase 24) |
| Audit log | Enterprise | Limited | Full tool + model metadata (Phase 25) |
| BYOK | Yes | N/A (Anthropic) | Yes |
| Self-hosted cloud | No | No | Documented path (Phase 24+) |

## Lessons to adopt

1. **Streaming UX** — token-by-token render, tool activity as first-class UI (not buried in markdown).
2. **Diff before apply** — show changes; never silent overwrite of large files.
3. **Mode clarity** — read-only Ask vs mutating Agent reduces accidents.
4. **Plan then build** — separate exploration from execution.

## Lessons to avoid

1. **Opaque tool failures** — always surface tool errors in UI and audit.
2. **Whole-repo context** — index and rank; never dump entire tree into prompt.
3. **Regex output surgery** — never silently delete model text (`code-old` anti-pattern).
4. **Fabricated tool calls** — if model refuses tools, re-prompt; do not inject fake `list_dir`.

The disposition of every implemented deprecated-app behavior is maintained in
the authoritative [legacy feature inventory](../01-product/06-legacy-feature-inventory.md).
This document describes market lessons; it is not a substitute for that
source-by-source migration decision record.

## Differentiation thesis

1. **Embedded Cersei** — no agent server hop; composable Rust core.
2. **Deny-by-default security** — enterprise-ready from Phase 7, not bolted on.
3. **Eury platform integration** — same auth, billing, projects, orgs as web app.
4. **Dual model path** — BYOK for latency; managed gateway for teams without key management.

## References

- [Cersei documentation](https://cersei.tryatlas.cc/docs)
- [Deprecated-app feature inventory](../01-product/06-legacy-feature-inventory.md)
