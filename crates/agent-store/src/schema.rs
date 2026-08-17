pub const V1_SCHEMA: &str = r"
-- Workspaces
CREATE TABLE workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Conversations
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_conversations_workspace_id ON conversations(workspace_id);

-- Messages
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);

-- Messages Full Text Search
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    conversation_id UNINDEXED,
    content='messages',
    content_rowid='rowid'
);

CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
  INSERT INTO messages_fts(rowid, content, conversation_id) VALUES (new.rowid, new.content, new.conversation_id);
END;
CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
  INSERT INTO messages_fts(messages_fts, rowid, content, conversation_id) VALUES('delete', old.rowid, old.content, old.conversation_id);
END;
CREATE TRIGGER messages_au AFTER UPDATE ON messages BEGIN
  INSERT INTO messages_fts(messages_fts, rowid, content, conversation_id) VALUES('delete', old.rowid, old.content, old.conversation_id);
  INSERT INTO messages_fts(rowid, content, conversation_id) VALUES (new.rowid, new.content, new.conversation_id);
END;

-- Runs
CREATE TABLE runs (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    mode TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    error TEXT
);
CREATE INDEX idx_runs_conversation_id ON runs(conversation_id);

-- Run Steps (Tool calls, tool results)
CREATE TABLE run_steps (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    step_type TEXT NOT NULL, -- 'ToolCall', 'ToolResult', etc.
    payload TEXT NOT NULL, -- JSON payload
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_run_steps_run_id ON run_steps(run_id);

-- Settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL, -- JSON
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Grants (Migrated from agent-policy)
CREATE TABLE grants (
    id TEXT PRIMARY KEY,
    tool_name TEXT NOT NULL,
    shape_hash TEXT NOT NULL,
    scope TEXT NOT NULL,
    expires_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE UNIQUE INDEX idx_grants_tool_shape ON grants(tool_name, shape_hash);

-- Audit Queue (Migrated from agent-policy)
CREATE TABLE audit_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    payload TEXT NOT NULL, -- JSON
    hash TEXT NOT NULL,
    prev_hash TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
";

/// Adds the workspace trust state the trust prompt records, so a trusted
/// workspace stays trusted across restarts instead of silently re-prompting.
/// Defaults to `untrusted`: a workspace predating this column has never had
/// the prompt answered, and the safe reading of "unknown" is read-only.
pub const V2_WORKSPACE_TRUST: &str = r"
ALTER TABLE workspaces ADD COLUMN trust_state TEXT NOT NULL DEFAULT 'untrusted'
    CHECK (trust_state IN ('trusted','untrusted'));
";
