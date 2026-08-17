# Indexing and Retrieval Specification

Spec-Version: 2.0.0

How the agent finds the right code without reading the whole repository. Retrieval quality is the single largest lever on answer quality, and retrieval cost is a major lever on latency and token spend ([ADR-0009](../02-architecture/adr/0009-index-and-retrieval-strategy.md)).

## Design decisions

| Decision | Rationale |
|---|---|
| The index is a **cache, never truth** | Every retrieval result is verified against the file on disk before it enters a prompt, so a stale index degrades ranking, never correctness |
| Lexical and structural first, embeddings optional | Exact-match search on code is strong, instant, offline, and free; embeddings are an add-on, not a dependency |
| Per-workspace database | Deleting a workspace deletes its index; one corrupt index cannot affect another project |
| Local embeddings by default | No code leaves the machine for indexing unless the user explicitly opts into a remote embedding provider |
| Build is interruptible and resumable | A first index of a large monorepo must never block the first prompt |

## Storage

```
$EURY_AGENT_DATA_DIR/index/<workspaceHash>/
  index.db          # files, symbols, chunks, FTS
  vectors.bin       # optional embedding store, memory-mapped
  manifest.json     # schema version, embedding model id, build state
```

Not encrypted, deliberately: it contains only data derived from plaintext files already on disk, and encryption would cost meaningful query latency for no threat-model gain. Secrets are never indexed (see exclusions).

### Schema

```sql
CREATE TABLE files (
  id            INTEGER PRIMARY KEY,
  path          TEXT NOT NULL UNIQUE,        -- workspace-relative, forward slashes
  language      TEXT,
  size_bytes    INTEGER NOT NULL,
  mtime_ms      INTEGER NOT NULL,
  content_sha256 TEXT NOT NULL,
  line_count    INTEGER NOT NULL,
  is_generated  INTEGER NOT NULL DEFAULT 0,
  is_test       INTEGER NOT NULL DEFAULT 0,
  indexed_at    INTEGER NOT NULL,
  parse_error   TEXT
);
CREATE INDEX idx_files_lang ON files(language);
CREATE INDEX idx_files_mtime ON files(mtime_ms DESC);

CREATE TABLE symbols (
  id          INTEGER PRIMARY KEY,
  file_id     INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  name        TEXT NOT NULL,
  kind        TEXT NOT NULL,      -- function|method|class|struct|enum|interface|type|const|module
  signature   TEXT,
  parent      TEXT,               -- enclosing class or module
  start_line  INTEGER NOT NULL, end_line INTEGER NOT NULL,
  is_exported INTEGER NOT NULL DEFAULT 0,
  doc         TEXT                -- leading doc comment, truncated to 500 chars
);
CREATE INDEX idx_symbols_name ON symbols(name);
CREATE INDEX idx_symbols_file ON symbols(file_id);

CREATE TABLE chunks (
  id          INTEGER PRIMARY KEY,
  file_id     INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  start_line  INTEGER NOT NULL, end_line INTEGER NOT NULL,
  token_count INTEGER NOT NULL,
  symbol_path TEXT,               -- "ClassName.methodName" for provenance
  content_sha256 TEXT NOT NULL,
  vector_row  INTEGER             -- offset into vectors.bin, NULL if not embedded
);
CREATE INDEX idx_chunks_file ON chunks(file_id, start_line);

CREATE VIRTUAL TABLE chunks_fts USING fts5(
  content, symbol_path, path UNINDEXED,
  tokenize = "unicode61 remove_diacritics 0 tokenchars '_$'"
);

CREATE TABLE imports (                 -- lightweight dependency edges
  file_id     INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
  target      TEXT NOT NULL,           -- raw specifier
  resolved_file_id INTEGER REFERENCES files(id) ON DELETE SET NULL
);
```

`tokenchars '_$'` keeps `snake_case`, `_private`, and `$scope` identifiers searchable as single tokens, which plain FTS5 tokenization would split and ruin for code.

## File discovery

Walk order and exclusions, applied in sequence:

| # | Rule |
|---|---|
| 1 | `.gitignore`, including nested files and negations, plus `.git/info/exclude` and the global gitignore |
| 2 | `.euryignore` (same syntax; the user's index-only exclusions) |
| 3 | Built-in directory excludes: `.git`, `node_modules`, `target`, `dist`, `build`, `out`, `.next`, `.venv`, `venv`, `__pycache__`, `.mypy_cache`, `.pytest_cache`, `vendor`, `Pods`, `.gradle`, `.terraform`, `coverage`, `.turbo`, `.cache` |
| 4 | Built-in file excludes: lockfiles, minified bundles (`*.min.js`, `*.min.css`), source maps, binaries, archives, media, fonts, `*.pb.go`, `*_pb2.py`, `*.generated.*` |
| 5 | **Secret-bearing paths are never indexed**: `.env*`, `*.pem`, `*.key`, `id_rsa*`, `.npmrc`, `.netrc`, `.aws/**`, `.ssh/**` — matching the tool catalog's always-deny list |
| 6 | Size limit: files above 2 MB are recorded as metadata only |
| 7 | Binary detection: NUL byte in the first 8 KB, or invalid UTF-8 |
| 8 | Symlinks are not followed outside the workspace; loops are detected by inode |
| 9 | Generated-file heuristic: a `@generated` or `DO NOT EDIT` marker in the first 5 lines sets `is_generated`, which halves the retrieval score |

Rule 5 is a security boundary, not an optimization. An indexed `.env` would leak into prompts through retrieval even though `read_file` refuses to open it.

## Symbol extraction

Tree-sitter, with a per-language query file.

| Tier | Languages |
|---|---|
| Tier 1 (full symbols, imports, docs) | TypeScript, JavaScript, TSX, Python, Rust, Go, Java, C#, C, C++, Ruby, PHP |
| Tier 2 (symbols only) | Kotlin, Swift, Scala, Dart, Lua, Elixir, Haskell, Zig |
| Tier 3 (headings and structure) | Markdown, JSON, YAML, TOML, SQL, HTML, CSS |
| Fallback | Line-based chunking with no symbols |

A parse failure is recorded in `files.parse_error` and the file falls back to line chunking. One malformed file never fails a build.

## Chunking

| Rule | Value |
|---|---|
| Target size | 400 tokens, hard max 800 |
| Boundaries | Prefer function, method, or class boundaries from tree-sitter; a small symbol becomes one chunk |
| Oversized symbols | Split at statement boundaries with a 40-token overlap and a repeated signature header |
| Small symbols | Adjacent siblings are merged up to the target size |
| Context header | Every chunk is prefixed with `path`, the enclosing symbol path, and the line range, so retrieved code is always attributable |
| Non-code | Markdown chunks split at heading boundaries; config files chunk at top-level keys |
| Determinism | Chunking is a pure function of content: identical content yields identical chunks, which makes embedding caching by hash effective |

## Embeddings (optional)

| Aspect | Decision |
|---|---|
| Default | Enabled with a bundled local model; no network calls |
| Local model | ~30 MB quantized, 384 dimensions, batch of 32, CPU-only |
| Remote option | The user may select a provider model, which requires explicit consent because chunk text leaves the machine, and is blocked entirely by air-gapped mode and by policy |
| Storage | `f32` (or `int8` quantized above 100 000 chunks) in `vectors.bin`, memory-mapped |
| Search | Exact cosine over the candidate set from stage 1, not a global ANN scan — the candidate set is already small, so exactness is affordable |
| Cache | Keyed by `content_sha256` + model id; unchanged chunks are never re-embedded |
| Model change | Invalidates all vectors and triggers a background rebuild; the `manifest.json` records the model id |
| Budget | Embedding work is capped at 25% of available cores and paused while a run is active |
| Failure | Retrieval degrades to lexical plus structural; it never blocks or errors |

## Retrieval pipeline

Two stages: cheap recall, then careful ranking.

```
Stage 1 — candidate generation (target <= 12 ms, up to 300 candidates)
  a. Query analysis: extract identifiers, quoted phrases, paths, error strings;
     detect intent (definition lookup | usage search | conceptual | error trace)
  b. Exact symbol match on extracted identifiers            (weight high)
  c. FTS5 match on chunk content and symbol paths
  d. Path match when the query looks like a path
  e. Vector search over embedded chunks, top 100            (when enabled)
  f. Recently-viewed and recently-edited files from the session
  g. Import-graph neighbors of files already in context

Stage 2 — ranking and selection
  score = 0.30 * lexical_bm25_normalized
        + 0.25 * symbol_exactness          (exact name > prefix > fuzzy)
        + 0.20 * semantic_cosine           (0 when embeddings are off, weights renormalize)
        + 0.10 * proximity                 (same directory/module as context files)
        + 0.08 * recency                   (git mtime, 30-day half-life)
        + 0.07 * centrality                (inbound import count, log-scaled)
  modifiers:
        x0.5  is_generated
        x0.7  is_test, unless the query is about tests
        x1.3  the chunk's file is explicitly attached by the user
        x1.5  the chunk contains the exact quoted phrase from the query
  then:
    - deduplicate overlapping chunks, keeping the larger span
    - cap at 3 chunks per file and 12 files total, to preserve breadth
    - expand each selected chunk to whole-symbol boundaries
    - verify each chunk against the file on disk (hash match); on mismatch, re-read
      from disk and reindex that file inline
    - fill the token budget in score order; stop at the budget
```

Intent detection changes the mix rather than the formula: a definition lookup weights symbol exactness far higher; a conceptual question weights semantics higher; an error-trace query prioritizes exact phrase matching on the error string.

### Budget

| Setting | Default |
|---|---|
| Retrieval share of the context window | 45% (see [runtime](01-agent-runtime-spec.md)) |
| Max files per retrieval | 12 |
| Max chunks per file | 3 |
| Min score to include | 0.15 |

Under-filling the budget is acceptable and often correct. Filling the window with weakly relevant code measurably degrades output quality and always costs money.

## Freshness

| Trigger | Behavior |
|---|---|
| Workspace open | Compare `mtime` and size against the manifest; enqueue changed files. The workspace is usable immediately |
| File watcher | Debounced 300 ms; a changed file is reindexed within 1 s |
| Rapid churn | Above 50 changes per second (branch switch, `npm install`), the watcher backs off to a periodic scan |
| Git operations | A HEAD change triggers a diff-driven targeted reindex, not a full rebuild |
| Retrieval-time check | Any returned chunk whose file hash has changed is reindexed inline before use |
| Periodic | A full consistency scan every 6 h while idle |
| Manual | `workspace_reindex { full: true }` |
| Schema or model change | The manifest version mismatch triggers a background rebuild; the old index keeps serving until the new one is ready |

## Performance targets

Reference hardware per the [benchmarks](../08-quality/03-performance-benchmarks.md).

| Operation | Target |
|---|---|
| First index, 1 000 files | < 6 s |
| First index, 10 000 files | < 60 s background, with search usable within 5 s |
| First index, 100 000 files | < 10 min, incremental availability throughout |
| Incremental single-file update | < 200 ms end to end |
| Stage 1 candidate generation | < 12 ms p95 |
| Full retrieval and rank | < 30 ms p95 (10 k files), < 80 ms (100 k) |
| Symbol lookup by exact name | < 5 ms |
| Local embedding throughput | ≥ 200 chunks/s |
| Index size | ≤ 12% of indexed source bytes without vectors, ≤ 25% with |
| Idle CPU while indexing | ≤ 25% of one core; 0% while a run is active |
| Memory during a full build | ≤ 300 MB |

Indexing yields to interactive work unconditionally. A background build must never make typing feel slow.

## Degraded modes

| Condition | Behavior |
|---|---|
| No index yet | Retrieval falls back to live `grep` + `glob`; the agent works from turn one |
| Index corrupt | Quarantine, rebuild in the background, fall back to live search meanwhile |
| Embeddings unavailable | Lexical plus structural only, weights renormalized; a subtle quality drop, never an error |
| Workspace too large | Above 200 000 files, prompt the user to scope indexing to subdirectories |
| Disk full | Indexing pauses and warns; the app keeps working |
| Air-gapped | Local embeddings only; remote embedding options are unregistered |

## Observability

Every retrieval records candidate count, selected chunks, files, tokens used, stage latencies, and whether embeddings contributed. The UI exposes this as "12 files retrieved, 8 400 tokens, 24 ms", expandable into the ranked list with per-chunk scores. This makes bad retrieval diagnosable by users instead of mysterious, and it is the input to retrieval-quality regression tests.

## Conformance tests

| ID | Test |
|---|---|
| T1 | Secret-bearing paths never appear in `files`, `chunks`, or any retrieval result |
| T2 | Gitignore semantics, including negations and nested files, match `git check-ignore` on a corpus |
| T3 | Chunking is deterministic and reproducible across platforms for identical content |
| T4 | A stale index never yields wrong content: modified files are detected and re-read at retrieval time |
| T5 | Retrieval quality suite: a labeled set of query-to-expected-file pairs holds recall@10 ≥ 0.85, checked per PR |
| T6 | Turning embeddings off changes ranking but keeps recall@10 ≥ 0.75 |
| T7 | All performance targets met on 1 k, 10 k, and 100 k file corpora |
| T8 | A corrupt index triggers quarantine plus rebuild with no user-visible error |
| T9 | Symlink loops, hostile unicode filenames, and 10 MB single-line files do not hang the walker |
| T10 | Deleting a workspace removes its entire index directory |
| T11 | A branch switch touching 5 000 files reindexes incrementally, not fully |
| T12 | Indexing is fully paused during an active run (CPU sampling assertion) |

## Related documents

- [ADR-0009](../02-architecture/adr/0009-index-and-retrieval-strategy.md)
- [Agent runtime](01-agent-runtime-spec.md)
- [Memory](08-memory-spec.md)
- [Tool catalog](02-tool-catalog-spec.md)
- [Performance benchmarks](../08-quality/03-performance-benchmarks.md)
