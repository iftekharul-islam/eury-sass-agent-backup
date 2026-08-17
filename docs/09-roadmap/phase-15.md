# Phase 15 — Code Intelligence

Spec-Version: 1.1.0

**Track:** D — Intelligence · **Estimated size:** 3 weeks · **Milestone:** —

## Goal

Index the workspace and retrieve only what matters, so context assembly is fast, cheap, and accurate on large repositories.

## Why this phase exists here

Without retrieval, every run either wastes tokens or misses the relevant file. This is the phase that determines whether the agent works on a real monorepo ([ADR-0009](../02-architecture/adr/0009-index-and-retrieval-strategy.md)).

## In scope

- Incremental indexer: file walk, ignore rules, content hashing, change detection
- Lexical index (trigram/inverted) for fast literal and regex search
- Symbol extraction via tree-sitter for the top languages
- Optional local embeddings with a bundled model; hybrid ranking
- Retrieval pipeline: query expansion, hybrid scoring, dedup, budget-aware packing
- Context assembly with explicit provenance shown in the UI
- Index lifecycle: background build, throttling, pause on battery, staleness handling
- Graceful degradation on very large repos and on unsupported languages

## Feature IDs

`F-060`, `F-061`

## Out of scope

- Cloud-hosted embeddings (not planned; local only)
- Cross-repository search

## Deliverables

| ID | Deliverable | Spec reference |
|---|---|---|
| D15.1 | Incremental indexer with watcher-driven updates | [indexing spec](../04-specs/09-indexing-and-retrieval-spec.md) |
| D15.2 | Lexical index powering `grep`/`glob` and palette search | [indexing spec](../04-specs/09-indexing-and-retrieval-spec.md) |
| D15.3 | Tree-sitter symbol extraction for the top 10 languages | [indexing spec](../04-specs/09-indexing-and-retrieval-spec.md) |
| D15.4 | Local embedding option with a hybrid ranker | [ADR-0009](../02-architecture/adr/0009-index-and-retrieval-strategy.md) |
| D15.5 | Retrieval pipeline with a token budget and dedup | [agent runtime](../04-specs/01-agent-runtime-spec.md) |
| D15.6 | Context panel showing exactly what was sent, with scores | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D15.7 | Index state UI: queued, scanning, embedding, ready, failed with reasons | [app shell](../05-ui/02-app-shell-and-navigation.md) |
| D15.8 | Degradation policy for 200k+ file repos | [offline and degraded modes](../02-architecture/06-offline-and-degraded-modes.md) |

## Key decisions and design notes

- Indexing is local only. Sending a repository to a hosted embedding service is incompatible with our privacy posture.
- Lexical search is the reliable floor; embeddings are an enhancement, and the product must be good without them.
- Retrieved context is always shown to the user with provenance — an agent that cites invisible context is unauditable.
- The index never gates chat; retrieval quality degrades until the index is ready.
- Indexing respects `.gitignore` plus a secret-file deny list, so `.env` files are not indexed at all.

## Contracts touched

- Index storage layout and version
- Retrieval result shape with scores and provenance
- Context assembly inputs and outputs

## Dependencies

- Phase 9 (storage)
- Phase 6 (file access through the sandbox)

## Risks

| Risk | Impact | Mitigation |
|---|---|---|
| Index build cost on large repos | Battery and CPU complaints | Throttling, battery awareness, incremental updates, configurable exclusions |
| Retrieval misses the right file | Poor answers | Eval categories specifically for retrieval; hybrid ranking; symbol-aware boosts |
| Embedding model size | Installer bloat | Optional download rather than bundled by default; lexical-only remains fully supported |
| Secrets indexed | Leak into prompts | Deny list plus secret-shape detection with redaction before assembly |

## Test plan

| Layer | Coverage |
|---|---|
| Unit | Ignore rules, hashing, chunking, ranking math |
| Integration | 1k/10k/50k/200k-file fixtures; incremental update correctness |
| Quality | Retrieval precision/recall on a labeled query set |
| Security | Secret files never appear in the index or in assembled context |
| Performance | Build, incremental update, and assembly benchmarks |

## Metrics and targets

| Metric | Target |
|---|---|
| Full index (10k files) | < 60 s background |
| Incremental update (1 file) | < 200 ms |
| Context assembly (10k files) | < 30 ms p95 |
| Context assembly (50k files) | < 80 ms p95 |
| Retrieval precision@5 on the labeled set | ≥ 0.8 |

## Exit criteria

- [ ] Index builds incrementally and survives restart
- [ ] Retrieval meets the precision target on the labeled query set
- [ ] Context panel shows every retrieved item with provenance
- [ ] Secret files are provably excluded
- [ ] Assembly latency targets met on 10k and 50k-file repos
- [ ] 200k-file repo degrades gracefully rather than failing

## Deferred from this phase

- LSP-quality cross-references (open question)
- Cross-repo retrieval (post-GA)

## Related documents

- [Roadmap overview](00-roadmap-overview.md)
- [Definition of done](../08-quality/05-definition-of-done.md)
- [Risk register](risk-register.md)
