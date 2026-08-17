# Local Data Model

Spec-Version: 2.1.0

The desktop's own SQLite database. It is the source of truth for conversations, runs, approvals, memory, and the audit queue. Cloud sync is optional and never required for correctness ([ADR-0004](../02-architecture/adr/0004-sqlite-local-store-and-encryption.md)).

## Location and files

```
$EURY_AGENT_DATA_DIR/                     # default: platform app-data dir, see ops config
  agent.db                                # main store (SQLCipher)
  agent.db-wal                            # write-ahead log
  agent.db-shm
  index/<workspaceHash>/index.db          # per-workspace code index (separate, disposable)
  memory/<workspaceHash>/memory.db        # per-workspace memory graph
  checkpoints/<runId>/                    # file snapshots (content-addressed)
  blobs/<sha256[0:2]>/<sha256>            # images and captured output
  logs/agent-YYYY-MM-DD.log
```

Rationale for splitting: `agent.db` must stay small, fast, and backed up conceptually forever; the index is large, rebuildable, and disposable. Losing the index costs a reindex. Losing `agent.db` costs history — so they get different durability and retention treatment.

## Encryption

| Aspect | Decision |
|---|---|
| Engine | SQLCipher 4, AES-256-CBC, HMAC-SHA512 page auth |
| Key | 32 random bytes generated on first launch |
| Key storage | OS keychain (`eury-agent` service, `db-key` account) |
| KDF | SQLCipher default (256 000 PBKDF2 iterations) |
| What is encrypted | `agent.db`, `memory.db`, and the blob store |
| What is not | The code index (contains only workspace-derived data that already exists in plaintext on disk) and logs (redacted by design) |
| Key unavailable | The app starts in a degraded read-only state and offers "reset local data" with an explicit warning — it never silently creates a second database |
| Key rotation | `PRAGMA rekey` behind a settings action, with a verified backup taken first |

Encryption protects the at-rest copy on a shared or backed-up disk. It does not protect against a compromised user account at runtime, and the [threat model](../03-security/01-threat-model.md) says so explicitly.

## Connection settings

```sql
PRAGMA journal_mode = WAL;        -- concurrent reads during writes
PRAGMA synchronous = NORMAL;      -- FULL for the audit queue transaction only
PRAGMA foreign_keys = ON;         -- enforced, not decorative
PRAGMA busy_timeout = 5000;
PRAGMA temp_store = MEMORY;
PRAGMA mmap_size = 268435456;     -- 256 MB
PRAGMA cache_size = -16000;       -- 16 MB
PRAGMA auto_vacuum = INCREMENTAL;
PRAGMA wal_autocheckpoint = 1000;
```

Access is through a single writer connection plus a read pool (default 4). All writes go through one Rust actor, which removes lock contention as a class of bug. Every multi-statement mutation runs in a transaction; there are no partial writes by construction.

## Conventions

| Convention | Rule |
|---|---|
| Ids | UUIDv7 as `TEXT`, so primary keys sort chronologically and index locality is good |
| Timestamps | `TEXT` RFC3339 UTC with milliseconds (`2026-08-16T09:00:00.123Z`) |
| Booleans | `INTEGER` 0/1 with a `CHECK` constraint |
| JSON | `TEXT` with `CHECK (json_valid(col))` |
| Enums | `TEXT` with an explicit `CHECK (col IN (...))` — the database rejects invalid states |
| Money | `INTEGER` micro-USD; floats never touch cost |
| Deletes | Hard deletes with `ON DELETE CASCADE`, except audit rows, which are immutable until uploaded |
| Naming | `snake_case` tables and columns; no table shares a name with a cloud Prisma model |

## Schema

### workspaces

```sql
CREATE TABLE workspaces (
  id             TEXT PRIMARY KEY,
  path           TEXT NOT NULL UNIQUE,
  path_hash      TEXT NOT NULL,                     -- sha256(canonical path), used in audit
  name           TEXT NOT NULL,
  trust_state    TEXT NOT NULL DEFAULT 'untrusted'
                 CHECK (trust_state IN ('untrusted','trusted','revoked')),
  trusted_at     TEXT,
  is_git_repo    INTEGER NOT NULL DEFAULT 0 CHECK (is_git_repo IN (0,1)),
  languages_json TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(languages_json)),
  settings_json  TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(settings_json)),
  last_opened_at TEXT,
  last_indexed_at TEXT,
  created_at     TEXT NOT NULL,
  updated_at     TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_workspaces_hash ON workspaces(path_hash);
CREATE INDEX idx_workspaces_recent ON workspaces(last_opened_at DESC);
```

### conversations

```sql
CREATE TABLE conversations (
  id           TEXT PRIMARY KEY,
  workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
  cloud_id     TEXT,                                -- set only when sync is enabled
  title        TEXT NOT NULL,
  mode         TEXT NOT NULL DEFAULT 'agent'
                 CHECK (mode IN ('chat','ask','plan','agent','build')),
  model        TEXT,
  pinned       INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0,1)),
  archived     INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0,1)),
  message_count INTEGER NOT NULL DEFAULT 0,
  total_cost_usd_micros INTEGER NOT NULL DEFAULT 0,
  last_message_at TEXT,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);
CREATE INDEX idx_conversations_ws ON conversations(workspace_id, last_message_at DESC);
CREATE INDEX idx_conversations_active ON conversations(archived, last_message_at DESC);
CREATE UNIQUE INDEX idx_conversations_cloud ON conversations(cloud_id) WHERE cloud_id IS NOT NULL;
```

### messages

```sql
CREATE TABLE messages (
  id              TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  run_id          TEXT REFERENCES runs(id) ON DELETE SET NULL,
  role            TEXT NOT NULL CHECK (role IN ('user','assistant','system','tool')),
  content         TEXT NOT NULL,
  reasoning       TEXT,                              -- kept separate; excluded from exports by default
  sort_order      INTEGER NOT NULL,
  turn            INTEGER,
  model           TEXT,
  trust           TEXT NOT NULL DEFAULT 'trusted' CHECK (trust IN ('trusted','untrusted')),
  compacted       INTEGER NOT NULL DEFAULT 0 CHECK (compacted IN (0,1)),
  replaced_by     TEXT REFERENCES messages(id) ON DELETE SET NULL,  -- compaction summary
  prompt_tokens   INTEGER, completion_tokens INTEGER,
  attachments_json TEXT CHECK (attachments_json IS NULL OR json_valid(attachments_json)),
  metadata_json   TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(metadata_json)),
  created_at      TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_messages_order ON messages(conversation_id, sort_order);
CREATE INDEX idx_messages_run ON messages(run_id);
```

`replaced_by` is how compaction stays non-destructive: the original messages remain, flagged, pointing at their summary. History can always be reconstructed.

### messages_fts

```sql
CREATE VIRTUAL TABLE messages_fts USING fts5(
  content,
  content='messages', content_rowid='rowid',
  tokenize='unicode61 remove_diacritics 2'
);
-- kept in sync by AFTER INSERT / UPDATE / DELETE triggers on messages
```

FTS5 with external content keeps the index small and avoids duplicating message text.

### runs

```sql
CREATE TABLE runs (
  id              TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  workspace_id    TEXT REFERENCES workspaces(id) ON DELETE SET NULL,
  parent_run_id   TEXT REFERENCES runs(id) ON DELETE SET NULL,
  resumed_from    TEXT REFERENCES runs(id) ON DELETE SET NULL,
  mode            TEXT NOT NULL CHECK (mode IN ('chat','ask','plan','agent','build')),
  role            TEXT,                              -- sub-agent role, NULL for top-level
  status          TEXT NOT NULL CHECK (status IN ('queued','assembling','streaming',
                    'tool_running','awaiting_approval','paused','compacting',
                    'complete','failed','limited','cancelled')),
  stop_reason     TEXT,
  prompt          TEXT NOT NULL,
  outcome_text    TEXT,
  model           TEXT NOT NULL,
  route           TEXT CHECK (route IN ('byok','managed')),
  policy_digest   TEXT NOT NULL,                     -- which policy governed this run
  turns           INTEGER NOT NULL DEFAULT 0,
  tool_call_count INTEGER NOT NULL DEFAULT 0,
  prompt_tokens   INTEGER NOT NULL DEFAULT 0,
  completion_tokens INTEGER NOT NULL DEFAULT 0,
  cached_tokens   INTEGER NOT NULL DEFAULT 0,
  cost_usd_micros INTEGER NOT NULL DEFAULT 0,
  cost_estimated  INTEGER NOT NULL DEFAULT 1 CHECK (cost_estimated IN (0,1)),
  timings_json    TEXT CHECK (timings_json IS NULL OR json_valid(timings_json)),
  error_code      TEXT, error_message TEXT,
  last_seq        INTEGER NOT NULL DEFAULT 0,        -- last emitted event seq, for resync
  started_at      TEXT NOT NULL,
  updated_at      TEXT NOT NULL,
  finished_at     TEXT
);
CREATE INDEX idx_runs_conversation ON runs(conversation_id, started_at DESC);
CREATE INDEX idx_runs_active ON runs(status) WHERE finished_at IS NULL;
CREATE INDEX idx_runs_workspace ON runs(workspace_id, started_at DESC);
CREATE INDEX idx_runs_parent ON runs(parent_run_id);
```

`idx_runs_active` is a partial index, which makes crash recovery ("find non-terminal runs") an instant lookup regardless of history size.

### run_steps

```sql
CREATE TABLE run_steps (
  id            TEXT PRIMARY KEY,
  run_id        TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
  seq           INTEGER NOT NULL,
  turn          INTEGER NOT NULL,
  kind          TEXT NOT NULL CHECK (kind IN ('tool','approval','compaction','note','subagent')),
  tool_call_id  TEXT,
  tool_name     TEXT,
  tool_class    TEXT CHECK (tool_class IS NULL OR tool_class IN
                  ('read','write','execute','network','mcp','write_outside_workspace')),
  status        TEXT NOT NULL CHECK (status IN ('running','ok','error','denied','cancelled','timeout')),
  args_hash     TEXT,                                -- sha256 of normalized args; args are not stored raw
  args_json     TEXT CHECK (args_json IS NULL OR json_valid(args_json)),  -- redacted
  result_summary TEXT,
  result_bytes  INTEGER,
  paths_json    TEXT CHECK (paths_json IS NULL OR json_valid(paths_json)),
  checkpoint_id TEXT REFERENCES checkpoints(id) ON DELETE SET NULL,
  exit_code     INTEGER,
  duration_ms   INTEGER,
  error_code    TEXT,
  created_at    TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_run_steps_seq ON run_steps(run_id, seq);
CREATE INDEX idx_run_steps_tool ON run_steps(tool_name, created_at DESC);
```

### approvals

```sql
CREATE TABLE approvals (
  id            TEXT PRIMARY KEY,
  run_id        TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
  tool_call_id  TEXT NOT NULL,
  tool_name     TEXT NOT NULL,
  tool_class    TEXT NOT NULL,
  risk          TEXT NOT NULL CHECK (risk IN ('low','medium','elevated','critical')),
  payload_json  TEXT NOT NULL CHECK (json_valid(payload_json)),   -- redacted approval payload
  decision      TEXT NOT NULL DEFAULT 'pending'
                  CHECK (decision IN ('pending','allow','deny','expired')),
  scope         TEXT CHECK (scope IS NULL OR scope IN ('once','run','session','always')),
  source        TEXT CHECK (source IS NULL OR source IN ('user','policy','timeout')),
  requested_at  TEXT NOT NULL,
  expires_at    TEXT NOT NULL,
  decided_at    TEXT,
  wait_ms       INTEGER
);
CREATE INDEX idx_approvals_pending ON approvals(decision) WHERE decision = 'pending';
CREATE UNIQUE INDEX idx_approvals_call ON approvals(run_id, tool_call_id);
```

### grants

```sql
CREATE TABLE grants (
  id            TEXT PRIMARY KEY,
  workspace_id  TEXT REFERENCES workspaces(id) ON DELETE CASCADE,
  scope         TEXT NOT NULL CHECK (scope IN ('run','session','always')),
  run_id        TEXT,                                -- set when scope = 'run'
  session_id    TEXT,                                -- set when scope = 'session'
  tool_name     TEXT NOT NULL,
  tool_class    TEXT NOT NULL,
  arg_pattern   TEXT,                                -- normalized path prefix or argv prefix
  granted_at    TEXT NOT NULL,
  granted_by    TEXT NOT NULL CHECK (granted_by IN ('user','policy')),
  expires_at    TEXT,
  revoked_at    TEXT,
  use_count     INTEGER NOT NULL DEFAULT 0,
  last_used_at  TEXT
);
CREATE INDEX idx_grants_lookup ON grants(workspace_id, tool_name, revoked_at);
```

Grants are narrow by construction: a tool name plus an argument pattern. `use_count` and `last_used_at` let the UI show "this grant has been used 47 times" so users can audit their own past generosity.

### checkpoints

```sql
CREATE TABLE checkpoints (
  id            TEXT PRIMARY KEY,
  run_id        TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
  workspace_id  TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  seq           INTEGER NOT NULL,
  label         TEXT,
  kind          TEXT NOT NULL CHECK (kind IN ('pre_tool','pre_step','manual')),
  file_count    INTEGER NOT NULL DEFAULT 0,
  total_bytes   INTEGER NOT NULL DEFAULT 0,
  git_head      TEXT, git_dirty INTEGER,
  restored_at   TEXT,
  created_at    TEXT NOT NULL
);

CREATE TABLE checkpoint_files (
  checkpoint_id TEXT NOT NULL REFERENCES checkpoints(id) ON DELETE CASCADE,
  path          TEXT NOT NULL,                       -- workspace-relative
  existed       INTEGER NOT NULL CHECK (existed IN (0,1)),
  content_sha256 TEXT,                               -- NULL when the file did not exist
  size_bytes    INTEGER,
  mode          INTEGER,
  eol           TEXT CHECK (eol IS NULL OR eol IN ('lf','crlf')),
  PRIMARY KEY (checkpoint_id, path)
);
```

Content is stored once per hash under `checkpoints/`, so repeated edits to one file cost one copy per distinct version, not one per tool call.

### memory

```sql
CREATE TABLE memory_entries (
  id           TEXT PRIMARY KEY,
  workspace_id TEXT REFERENCES workspaces(id) ON DELETE CASCADE,   -- NULL = user-global
  kind         TEXT NOT NULL CHECK (kind IN ('preference','project_fact','code_pattern',
                 'decision','constraint','glossary')),
  text         TEXT NOT NULL,
  source       TEXT NOT NULL CHECK (source IN ('user','extracted','imported','policy')),
  source_run_id TEXT REFERENCES runs(id) ON DELETE SET NULL,
  confidence   REAL NOT NULL DEFAULT 1.0 CHECK (confidence BETWEEN 0 AND 1),
  pinned       INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0,1)),
  use_count    INTEGER NOT NULL DEFAULT 0,
  last_used_at TEXT,
  superseded_by TEXT REFERENCES memory_entries(id) ON DELETE SET NULL,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL
);
CREATE INDEX idx_memory_scope ON memory_entries(workspace_id, kind) WHERE superseded_by IS NULL;

CREATE TABLE memory_proposals (
  id           TEXT PRIMARY KEY,
  workspace_id TEXT REFERENCES workspaces(id) ON DELETE CASCADE,
  run_id       TEXT REFERENCES runs(id) ON DELETE SET NULL,
  kind         TEXT NOT NULL,
  text         TEXT NOT NULL,
  confidence   REAL NOT NULL,
  status       TEXT NOT NULL DEFAULT 'pending'
                 CHECK (status IN ('pending','accepted','rejected','expired')),
  created_at   TEXT NOT NULL,
  decided_at   TEXT
);
```

### audit_queue

```sql
CREATE TABLE audit_queue (
  id            TEXT PRIMARY KEY,
  seq           INTEGER NOT NULL UNIQUE,             -- device-local monotonic chain
  prev_hash     TEXT NOT NULL,
  event_hash    TEXT NOT NULL,
  event_type    TEXT NOT NULL,
  payload_json  TEXT NOT NULL CHECK (json_valid(payload_json)),
  org_id        TEXT,
  occurred_at   TEXT NOT NULL,
  created_at    TEXT NOT NULL,
  attempts      INTEGER NOT NULL DEFAULT 0,
  last_error    TEXT,
  uploaded_at   TEXT
);
CREATE INDEX idx_audit_pending ON audit_queue(uploaded_at, seq) WHERE uploaded_at IS NULL;
```

The hash chain (`prev_hash` + `event_hash`) makes local tampering and gaps detectable server-side ([audit and retention](../06-enterprise/04-audit-and-retention.md)). Inserts into this table use `synchronous = FULL`, because an audit record that was reported as written must survive a power loss.

### settings, secrets metadata, sync

```sql
CREATE TABLE settings (
  key        TEXT PRIMARY KEY,
  value_json TEXT NOT NULL CHECK (json_valid(value_json)),
  scope      TEXT NOT NULL DEFAULT 'user' CHECK (scope IN ('user','workspace','machine')),
  locked     INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0,1)),   -- set by machine policy
  updated_at TEXT NOT NULL
);

CREATE TABLE secret_refs (                          -- metadata only; values live in the keychain
  name           TEXT PRIMARY KEY,                  -- e.g. 'provider:openai'
  present        INTEGER NOT NULL CHECK (present IN (0,1)),
  last_verified_at TEXT,
  created_at     TEXT NOT NULL
);

CREATE TABLE policy_cache (
  id           TEXT PRIMARY KEY CHECK (id = 'current'),
  digest       TEXT NOT NULL,
  document_json TEXT NOT NULL CHECK (json_valid(document_json)),
  signature    TEXT,
  fetched_at   TEXT NOT NULL,
  expires_at   TEXT NOT NULL
);

CREATE TABLE sync_state (
  entity      TEXT PRIMARY KEY,                     -- 'conversations' | 'memory' | ...
  cursor      TEXT,
  last_sync_at TEXT,
  last_error  TEXT
);

CREATE TABLE schema_migrations (
  version     INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,
  checksum    TEXT NOT NULL,
  applied_at  TEXT NOT NULL,
  duration_ms INTEGER NOT NULL
);
```

`locked` on `settings` is how MDM-delivered machine configuration wins over user preference without a second storage system.

## Cross-table invariants

| # | Invariant | Enforced by |
|---|---|---|
| V1 | At most one run per conversation in a non-terminal state | Partial unique index + writer actor |
| V2 | `messages.sort_order` is unique and gapless per conversation | Unique index + single writer |
| V3 | Every `run_steps` row with `tool_class` in (`write`,`execute`) has a `checkpoint_id` or an explicit `no_checkpoint` reason | Insert-time assertion + test T5 |
| V4 | A resolved approval is never mutated again | Trigger rejecting updates when `decision != 'pending'` |
| V5 | `audit_queue.seq` is contiguous from 1 | Insert path + startup verification |
| V6 | `conversations.message_count` equals the actual row count | Triggers, verified by `integrity_check` |
| V7 | No orphan checkpoint directories | Startup reconciliation sweep |
| V8 | A terminal run has `finished_at` set | Trigger |

## Migrations

| Rule | Detail |
|---|---|
| Numbering | `NNN_description.sql`, sequential, never renumbered |
| Direction | Forward-only. No down migrations; recovery is restore-from-backup |
| Atomicity | Each migration runs in one transaction with `foreign_keys = OFF` during table rebuilds |
| Checksum | Recorded; a changed checksum for an applied migration aborts startup with `EURY_STORE_MIGRATION_TAMPERED` |
| Backup | Before any migration, `agent.db` is copied to `agent.db.pre-<version>`; kept until the next successful launch |
| Failure | Transaction rolls back, the backup is restored, and the app starts in read-only mode with a clear repair action |
| Newer schema | A database newer than the binary is refused (`EURY_STORE_SCHEMA_TOO_NEW`) — a downgraded app never writes to a newer schema |
| Testing | Every migration is tested against a seeded database from each prior released version |

## Retention and maintenance

| Data | Default retention | Configurable |
|---|---|---|
| Conversations and messages | Forever | Yes, per workspace |
| Runs and run steps | Forever (metadata is small) | Yes |
| Checkpoint file contents | 14 days, or 2 GB, or 100 runs per workspace, whichever comes first | Yes |
| Blobs | Reference-counted; deleted with their message | No |
| Audit queue (uploaded) | 7 days, then pruned | Enterprise: keep until confirmed |
| Audit queue (pending) | Never pruned; a full queue blocks privileged tools rather than dropping records | No |
| Logs | 7 files, 10 MB each, rotated | Yes |
| Index | Rebuilt on demand; pruned when a workspace is removed | Yes |

Maintenance on idle: `PRAGMA incremental_vacuum` weekly, `ANALYZE` after 10 000 writes, `PRAGMA wal_checkpoint(TRUNCATE)` on clean shutdown, `PRAGMA integrity_check` monthly and after any crash.

## Performance targets

| Operation | p95 |
|---|---|
| Open database and verify schema | 30 ms |
| Insert message | 2 ms |
| Load last 50 messages of a conversation | 15 ms |
| List 100 conversations with counts | 20 ms |
| FTS search across 100 000 messages | 100 ms |
| Insert run step + audit row (one transaction) | 5 ms |
| Crash-recovery scan for active runs | 10 ms |
| Database size, 10 000 messages | < 60 MB |

## Corruption handling

Detection: `integrity_check` failure, an `SQLITE_CORRUPT` result, or a checksum mismatch. Response, in order: retry once after `wal_checkpoint`; attempt `.recover` into a fresh file; on success, swap in the recovered database and report how many rows were lost; on failure, quarantine the file as `agent.db.corrupt-<timestamp>`, start a fresh database, and tell the user exactly where the quarantined file is. The app never deletes a corrupt database, because it may still be recoverable by hand.

## Import from the deprecated app

One-time, opt-in, non-destructive ([migration map](../00-overview/05-naming-and-migration-map.md)).

| Legacy source | Target | Rule |
|---|---|---|
| `~/.eury-code/sessions.json` | `conversations` + `messages` | Titles preserved; imported entries tagged `imported: true` |
| `~/.eury-code/settings.json` | `settings` | Only keys that still exist; unknown keys reported, not migrated |
| `~/.eury-code/auth.json` | nothing | Never imported; the user re-authenticates |
| Legacy API keys | nothing | Never copied; the user re-enters them into the keychain |
| Legacy plans | `.eury/plans/` | Converted to the current [plan format](11-plan-format-spec.md) |

The legacy directory is left untouched so the old app keeps working until the user removes it. Import is idempotent: running it twice does not duplicate conversations.

## Conformance tests

| ID | Test |
|---|---|
| T1 | Fresh database creates and passes `integrity_check`; the key lands in the keychain |
| T2 | Every `CHECK` constraint rejects at least one invalid value (generated test) |
| T3 | Migration chain applies cleanly from every released version's seeded database |
| T4 | A tampered migration checksum aborts startup |
| T5 | Every write/execute step has a checkpoint (V3) |
| T6 | Kill -9 during a run leaves a consistent, recoverable state at every persistence point |
| T7 | Audit inserts survive simulated power loss (`synchronous = FULL` path) |
| T8 | A deliberately corrupted page triggers recovery and quarantine, never silent data loss |
| T9 | Cascade deletes leave no orphan rows, blobs, or checkpoint directories |
| T10 | Performance targets met on a 100 000-message seeded database |
| T11 | The database file is unreadable by plain `sqlite3` without the key |
| T12 | Legacy import is idempotent and never imports credentials |

## Related documents

- [ADR-0004](../02-architecture/adr/0004-sqlite-local-store-and-encryption.md)
- [Checkpoints and rollback](12-checkpoint-and-rollback-spec.md)
- [Memory](08-memory-spec.md)
- [Audit and retention](../06-enterprise/04-audit-and-retention.md)
- [Backup and DR](../07-ops/07-backup-and-dr.md)
