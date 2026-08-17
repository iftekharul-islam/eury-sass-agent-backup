# ADR-0009: Index and Retrieval Strategy

Spec-Version: 1.0.0

**Status:** Accepted  
**Date:** 2026-08-16

## Context

Sending entire repositories to the LLM is slow, expensive, and hits context limits. `code-old` had no indexer.

## Decision

**Hybrid retrieval:**

1. **Incremental file index** — paths, mtime, size, language, gitignore-aware.
2. **Symbol index** — tree-sitter (or LSP) for definitions/references where available.
3. **Embedding index (Phase 15+)** — chunk embeddings stored locally; optional cloud embedding API.
4. **Ranking** — combine lexical (grep), symbol match, embedding similarity, recency, open files.

Context budget enforced in `agent-core` before each model call. Target: < 32k tokens context assembly locally in < 30 ms p95.

## Consequences

**Positive:**
- Scalable to large monorepos.
- Predictable token costs.

**Negative:**
- Index build CPU on workspace open.
- Embedding storage disk use.

**Mitigations:**
- Background indexing with progress UI.
- `.euryignore` for vendor dirs.
