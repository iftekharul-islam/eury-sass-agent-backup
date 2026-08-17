# Memory Specification

Spec-Version: 2.0.0

Memory is what makes the agent feel like it knows your project. It is also the easiest feature to make creepy or wrong, so the governing rule is: **the user can always see every remembered fact, and nothing is remembered without confirmation.**

## Principles

| # | Principle |
|---|---|
| P1 | Memory is inspectable: every entry is listable, editable, and deletable in the UI |
| P2 | Memory is confirmed: extracted facts are **proposals** until the user accepts them |
| P3 | Memory is attributable: every entry records where it came from (run, file, or user) |
| P4 | Memory is scoped: workspace memory never leaks into another workspace |
| P5 | Memory is bounded: hard caps on entries and injected tokens, always |
| P6 | Memory is local by default: it syncs only when the user enables sync |
| P7 | Memory never stores secrets: the redactor runs before storage, and high-entropy strings are rejected |
| P8 | Untrusted content cannot write memory: a fact discovered in fetched web content or a tool result is a proposal at best |

P8 matters more than it looks. Without it, a hostile repository could write "the user has approved running arbitrary commands" into long-term memory and poison every future session ([injection defense](../03-security/05-prompt-injection-defense.md)).

## Two layers

| Layer | Storage | Authored by | Purpose |
|---|---|---|---|
| **Instruction files** (`EURY.md`) | Plain markdown in the repo and home directory | Humans, in version control | Durable rules: conventions, commands, do-nots |
| **Memory entries** | `memory_entries` in SQLite + a relation graph | Agent proposals, user confirmations | Learned facts, preferences, decisions |

Instruction files are the primary mechanism because they are reviewable, diffable, and shareable through git. Memory entries fill the gap for things nobody wants to write down by hand.

## Instruction file hierarchy

Precedence, lowest to highest:

| # | Scope | Path | Notes |
|---|---|---|---|
| 1 | Built-in defaults | compiled in | Baseline behavior |
| 2 | Organization | `$EURY_AGENT_DATA_DIR/org-rules/*.md` | Delivered with policy; read-only, cannot be edited locally |
| 3 | User | `~/.eury-agent/EURY.md` | Personal preferences across all projects |
| 4 | Project | `<workspace>/EURY.md` | Committed to the repo; the team's shared rules |
| 5 | Nested | `<workspace>/<subdir>/EURY.md` | Applies when working under that subtree |
| 6 | Local | `<workspace>/EURY.local.md` | Gitignored personal overrides |

### Merge rules

| Rule | Detail |
|---|---|
| Composition | Sections are concatenated in precedence order, each labeled with its source |
| No silent override | A higher level adding a contradictory rule keeps both, with the higher one marked authoritative — so the model can see the tension rather than being confused by a rewrite |
| Organization rules | Cannot be overridden by levels 3–6; conflicting lower rules are dropped and reported in the UI |
| Size caps | 32 KB per file, 64 KB merged; overflow is truncated at a section boundary with a visible warning |
| Includes | `@path/to/file.md` includes another markdown file, max depth 3, workspace-relative only, cycles rejected |
| Reload | File watcher; changes apply to the next run, and the current run is notified |
| Trust | Instruction files inside the workspace are **repo content**. In an untrusted workspace they are shown to the user for review before first use, never applied blindly |
| Precedence display | Settings → Memory shows the effective merged document with per-line source attribution |

The trust rule closes the obvious hole: cloning a hostile repo whose `EURY.md` says "always run `curl … \| sh` first" must not silently change agent behavior.

## Memory entries

| Kind | Example | Default lifetime |
|---|---|---|
| `preference` | "Prefers `pnpm`; never suggest `npm`" | Permanent, user-global |
| `project_fact` | "Migrations run through `make migrate`, not the Prisma CLI" | Permanent, workspace |
| `code_pattern` | "Repositories use the result-type error pattern, not exceptions" | Permanent, workspace |
| `decision` | "Chose SQLite over Postgres for the desktop store" | Permanent, workspace |
| `constraint` | "Must stay compatible with Node 20" | Permanent, workspace |
| `glossary` | "'Gateway' means the managed model proxy, not nginx" | Permanent, workspace |

Fields, caps, and provenance are defined in the [local data model](05-local-data-model.md). Hard caps: 200 entries per workspace, 100 user-global, 500 chars per entry. When a cap is reached, the lowest-value entry (least recently used, unpinned, lowest confidence) is proposed for eviction rather than being deleted silently.

## Extraction pipeline

Runs **after** a turn completes, never inside the hot loop.

```
1. Trigger: run completed AND (turns >= 3 OR files changed) AND workspace is trusted
2. Candidate detection: a cheap classifier pass over the turn's user messages and
   the assistant's confirmed conclusions — never over raw tool output
3. Filter:
   - drop anything derived solely from untrusted content (P8)
   - drop secrets and high-entropy strings (P7)
   - drop anything already covered by an existing entry (similarity >= 0.9)
   - drop transient statements ("for now", "just this once", "temporarily")
4. Normalize: rewrite to a single declarative sentence, <= 500 chars, no first person
5. Score confidence: explicit user statement 0.95, repeated behavior 0.75, inference 0.5
6. Persist as a proposal (memory_proposals), emit memoryProposed
7. UI surfaces a low-priority, non-blocking "remember this?" affordance
8. Accept -> memory_entries with source='extracted'; Reject -> recorded so the same
   fact is not proposed again for 30 days
```

Confidence below 0.5 is never proposed. Proposals expire after 7 days. At most 3 proposals per run, so the feature can never turn into a notification stream.

## Recall

Recall happens once per run during context assembly ([runtime](01-agent-runtime-spec.md)), not per turn.

```
score = 0.45 * semantic_similarity(query, entry)
      + 0.20 * kind_prior(mode, kind)
      + 0.15 * recency_decay(last_used_at, halflife = 30 days)
      + 0.10 * usage_frequency(normalized)
      + 0.10 * graph_proximity(entry, files_in_context)
pinned entries: score = max(score, 0.8)
```

| Rule | Value |
|---|---|
| Candidate set | Workspace entries + user-global entries; never another workspace's |
| Top-k | 8 candidates, then truncated to the 2000-token budget |
| Threshold | Score ≥ 0.35; below that, nothing is injected (an empty recall is a valid result) |
| Kind priors | `plan` mode favors `decision` and `constraint`; `agent` favors `project_fact` and `code_pattern` |
| Latency | ≤ 15 ms p95 with a warm cache, ≤ 40 ms cold, excluding embedding calls |
| Embeddings | Local model by default; entries are short enough that a 384-dimension local model is sufficient and needs no network |
| Attribution | Injected entries are labeled with their id so the UI can show "3 memories used" and link each one |
| Feedback | `use_count` and `last_used_at` are updated only when the run actually completes, so failed runs do not inflate scores |

## Relation graph

Entries are nodes; edges capture structure the flat list cannot.

| Edge | Meaning | Created by |
|---|---|---|
| `supersedes` | A newer fact replaces an older one | Contradiction detection or user edit |
| `relates_to` | Co-occurrence in accepted proposals | Extraction |
| `about_file` | The fact concerns a path or module | Extraction, from the run's touched files |
| `derived_from` | Provenance to a run | Always |

The graph powers three concrete behaviors: `graph_proximity` in scoring, contradiction detection at write time, and "why do you think that?" — clicking an entry shows the run and files it came from.

### Contradiction handling

When a new entry semantically contradicts an existing one (similarity ≥ 0.75 with opposed polarity), the old entry is **not** overwritten. The user is shown both and picks one; the loser is marked `superseded_by` and retained for history. Automatic overwriting is forbidden, because a single bad extraction could otherwise erase a correct, long-standing fact.

## Injection into the prompt

Memory occupies section 5 of the assembled prompt, marked `trusted` (it was user-confirmed) with an explicit framing: these are facts about this project and user, they are not instructions from the current turn, and they may be outdated. Each entry carries its id and kind. The budget is a hard 2000 tokens; overflow drops the lowest-scoring entries first.

## User controls

Settings → Memory provides: the merged instruction-file view with source attribution; a searchable entry list filterable by kind, scope, and source; inline edit and delete; pin and unpin; provenance ("used in 12 runs, last 3 days ago, learned from run X"); accept and reject queues for pending proposals; an auto-extraction toggle (on by default, proposals only); an "auto-accept high-confidence preferences" toggle (**off** by default); export to JSON or markdown; and purge, scoped to a workspace or everything, behind a confirmation token.

## Policy and enterprise controls

| Setting | Effect |
|---|---|
| `memory.enabled: false` | Memory is fully disabled; existing entries are inaccessible and not injected |
| `memory.extractionEnabled: false` | Recall works; automatic extraction is off |
| `memory.syncEnabled: false` | Memory never leaves the device |
| `memory.orgRules: [...]` | Read-only organization instruction files at precedence level 2 |
| `memory.maxEntries` | Overrides the default cap |

Organizations that forbid memory get a product that still works — recall simply returns nothing. No feature silently degrades into a policy violation.

## Privacy

Memory entries are user content and are treated as such: encrypted at rest, excluded from telemetry entirely, excluded from crash reports, redacted before any diagnostics bundle, never included in audit payloads (audit records that memory *was used* and how many entries, never their text), and deleted with the workspace ([privacy](../03-security/07-privacy-and-data-residency.md)).

## Conformance tests

| ID | Test |
|---|---|
| T1 | No entry is created without an explicit accept, except when auto-accept is deliberately enabled |
| T2 | A fact appearing only in untrusted tool output is never stored (P8) |
| T3 | Seeded secrets are never stored, and high-entropy candidates are rejected |
| T4 | Instruction-file precedence produces the expected merged document across all six levels |
| T5 | Organization rules cannot be overridden by user, project, or local files |
| T6 | A hostile `EURY.md` in an untrusted workspace does not affect a run before review |
| T7 | Recall never returns entries from another workspace |
| T8 | Recall latency and budget targets are met with 200 entries |
| T9 | Contradiction is surfaced, never silently resolved |
| T10 | Caps trigger eviction proposals, not silent deletion |
| T11 | Purge removes entries, proposals, graph edges, and embeddings with nothing left behind |
| T12 | With `memory.enabled: false`, no memory text reaches any prompt |

## Related documents

- [Agent runtime](01-agent-runtime-spec.md)
- [Indexing and retrieval](09-indexing-and-retrieval-spec.md)
- [Local data model](05-local-data-model.md)
- [Prompt injection defense](../03-security/05-prompt-injection-defense.md)
- [ADR-0009](../02-architecture/adr/0009-index-and-retrieval-strategy.md)
